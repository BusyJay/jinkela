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
        self.internal_build(&proto_dir);
        let modules: Vec<_> = std::fs::read_dir(&proto_dir).unwrap().map(|res| {
            let path = match res {
                Ok(e) => e.path(),
                Err(e) => panic!("failed to list {}: {:?}", proto_dir, e),
            };
            let name = path.file_stem().unwrap().to_str().unwrap();
            name.replace('-', "_")
        }).collect();
        let mut f = File::create(format!("{}/mod.rs", proto_dir)).unwrap();
        for module in &modules {
            writeln!(f, "pub mod {};", module).unwrap();
        }
    }

    #[cfg(feature = "protobuf-codec")]
    fn internal_build(&self, out_dir: &str) {
        println!("building protobuf at {}", out_dir);
        let mut includes: Vec<&str> = Vec::new();
        for i in &self.includes {
            includes.push(&i);
        }
        let mut inputs: Vec<&str> = Vec::new();
        for s in &self.sources {
            inputs.push(&s);
        }
        protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
            out_dir: out_dir,
            includes: &includes,
            input: &inputs,
            customize: protobuf_codegen_pure::Customize::default(),
        }).unwrap();
        self.build_grpcio(&includes, &inputs, &out_dir);
    }

    #[cfg(feature = "prost-codec")]
    fn internal_build(&self, out_dir: &str) {
        println!("building prost at {}", out_dir);
        let mut cfg = prost_build::Config::new();
        cfg.type_attribute(".", "#[derive(::jinkela::Classicalize)]").out_dir(out_dir);
        self.config_grpcio(&mut cfg);
        cfg.compile_protos(&self.sources, &self.includes).unwrap();
    }

    #[cfg(feature = "grpcio-protobuf-codec")]
    fn build_grpcio(&self, includes: &[&str], inputs: &[&str], output: &str) {
        println!("building protobuf with grpcio at {}", output);
        let output_dir = std::path::Path::new(output);
        let protos = protobuf_codegen_pure::parse_and_typecheck(&includes, &inputs).unwrap();
        println!("{:?}", protos.file_descriptors);
        println!("{:?}", protos.relative_paths);
        let results = grpcio_compiler::codegen::gen(&protos.file_descriptors, &protos.relative_paths);
        for res in results {
            println!("writing {}", res.name);
            let out_file = output_dir.join(&res.name);
            let mut f = File::create(&out_file).unwrap();
            f.write_all(&res.content).unwrap();
        }
    }

    #[cfg(all(feature = "protobuf-codec", not(feature = "grpcio-protobuf-codec")))]
    fn build_grpcio(&self, _includes: &[&str], _inputs: &[&str], _output: &str) {}

    #[cfg(feature = "grpcio-prost-codec")]
    fn config_grpcio(&self, cfg: &mut prost_build::Config) {
        println!("building prost with grpcio");
        cfg.service_generator(Box::new(grpcio_compiler::prost_codegen::Generator));
    }

    #[cfg(all(feature = "prost-codec", not(feature = "grpcio-prost-codec")))]
    fn config_grpcio(&self, _cfg: &mut prost_build::Config) {}
}
