use std::fs::File;
use std::io::Write;

#[derive(Default)]
pub struct Builder {
    out_dir: Option<String>,
    includes: Vec<String>,
    sources: Vec<String>,
}

impl Builder {
    pub fn out_dir(&mut self, dir: impl Into<String>) -> &mut Builder {
        self.out_dir = Some(dir.into());
        self
    }

    pub fn include_dir(&mut self, dir: impl Into<String>) -> &mut Builder {
        self.includes.push(dir.into());
        self
    }

    pub fn compile_proto(&mut self, proto: impl Into<String>) -> &mut Builder {
        self.sources.push(proto.into());
        self
    }

    pub fn build(&self) {
        for (key, value) in std::env::vars() {
            println!("{}: {}", key, value);
        }
        let proto_dir = self.out_dir.clone().unwrap_or_else(|| {
            let out_dir = std::env::var("OUT_DIR").unwrap();
            format!("{}/protos", out_dir)
        });
        if std::path::Path::new(&proto_dir).exists() {
            std::fs::remove_dir_all(&proto_dir).unwrap();
        }
        std::fs::create_dir_all(&proto_dir).unwrap();

        let protoc = protoc::Protoc::from_env_path();
        let desc_file = format!("{}/mod.desc", proto_dir);
        let mut includes: Vec<&str> = Vec::new();
        for i in &self.includes {
            includes.push(&i);
        }
        let mut inputs: Vec<&str> = Vec::new();
        for s in &self.sources {
            inputs.push(&s);
        }
        protoc.write_descriptor_set(protoc::DescriptorSetOutArgs {
            out: &desc_file,
            includes: &includes,
            input: &inputs,
            include_imports: true,
        }).unwrap();

        self.internal_build(&proto_dir, &desc_file);
        let modules: Vec<_> = std::fs::read_dir(&proto_dir).unwrap().filter_map(|res| {
            let path = match res {
                Ok(e) => e.path(),
                Err(e) => panic!("failed to list {}: {:?}", proto_dir, e),
            };
            if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                let name = path.file_stem().unwrap().to_str().unwrap();
                Some((name.replace('-', "_"), name.to_owned()))
            } else {
                None
            }
        }).collect();
        let mut f = File::create(format!("{}/mod.rs", proto_dir)).unwrap();
        for (module, file_name) in &modules {
            if !module.contains('.') {
                writeln!(f, "pub mod {};", module).unwrap();
                continue;
            }
            let mut level = 0;
            for part in module.split('.') {
                writeln!(f, "{:level$}pub mod {} {{", "", part, level = level).unwrap();
                level += 1;
            }
            writeln!(f, "include!(\"{}.rs\");", file_name).unwrap();
            for _ in (0..level).rev() {
                writeln!(f, "{:1$}}}", "", level).unwrap();
            }
        }
    }

    #[cfg(feature = "protobuf-codec")]
    fn internal_build(&self, out_dir: &str, desc_file: &str) {
        println!("building protobuf at {} for {}", out_dir, desc_file);
        
        let desc_bytes = std::fs::read(&desc_file).unwrap();
        let desc: protobuf::descriptor::FileDescriptorSet = protobuf::parse_from_bytes(&desc_bytes).unwrap();
        let mut files_to_generate = Vec::new();
        'outer: for file in &self.sources {
            let f = std::path::Path::new(file);
            for include in &self.includes {
                if let Some(truncated) = f.strip_prefix(include).ok() {
                    files_to_generate.push(format!("{}", truncated.display()));
                    continue 'outer;
                }
            }

            panic!("file {:?} is not found in includes {:?}", file, self.includes);
        }

        protobuf_codegen::gen_and_write(
            desc.get_file(),
            &files_to_generate,
            &std::path::Path::new(out_dir),
            &protobuf_codegen::Customize::default(),
        ).unwrap();
        self.build_grpcio(&desc.get_file(), &files_to_generate, &out_dir);
    }

    #[cfg(feature = "prost-codec")]
    fn internal_build(&self, out_dir: &str, desc_file: &str) {
        println!("building prost at {}", out_dir);
        let mut cfg = prost_build::Config::new();
        cfg.type_attribute(".", "#[derive(::jinkela::Classicalize)]").out_dir(out_dir);
        cfg.compile_protos(&self.sources, &self.includes).unwrap();

        self.build_grpcio(out_dir, desc_file);
    }

    #[cfg(feature = "grpcio-protobuf-codec")]
    fn build_grpcio(&self, desc: &[protobuf::descriptor::FileDescriptorProto], files_to_generates: &[String], output: &str) {
        println!("building protobuf with grpcio at {}", output);
        let output_dir = std::path::Path::new(output);
        let results = grpcio_compiler::codegen::gen(&desc, &files_to_generates);
        for res in results {
            let out_file = output_dir.join(&res.name);
            let mut f = File::create(&out_file).unwrap();
            f.write_all(&res.content).unwrap();
        }
    }

    #[cfg(all(feature = "protobuf-codec", not(feature = "grpcio-protobuf-codec")))]
    fn build_grpcio(&self, _: &[protobuf::descriptor::FileDescriptorProto], _: &[String], _: &str) {}

    #[cfg(feature = "grpcio-prost-codec")]
    fn build_grpcio(&self, out_dir: &str, desc_file: &str) {
        use prost::Message;
        
        let desc_bytes = std::fs::read(&desc_file).unwrap();
        let desc = prost_types::FileDescriptorSet::decode(&desc_bytes).unwrap();
        let mut files_to_generate = Vec::new();
        'outer: for file in &self.sources {
            let f = std::path::Path::new(file);
            for include in &self.includes {
                if let Some(truncated) = f.strip_prefix(include).ok() {
                    files_to_generate.push(format!("{}", truncated.display()));
                    continue 'outer;
                }
            }

            panic!("file {:?} is not found in includes {:?}", file, self.includes);
        }
        let out_dir = std::path::Path::new(out_dir);
        let results = grpcio_compiler::codegen::gen(&desc.file, &files_to_generate);
        for res in results {
            let out_file = out_dir.join(&res.name);
            let mut f = File::create(&out_file).unwrap();
            f.write_all(&res.content).unwrap();
        }
    }

    #[cfg(all(feature = "prost-codec", not(feature = "grpcio-prost-codec")))]
    fn build_grpcio(&self, _out_dir: &str, _desc_file: &str) {}
}
