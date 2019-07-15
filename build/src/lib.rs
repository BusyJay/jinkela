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
    }

    #[cfg(feature = "prost-codec")]
    fn internal_build(&self, out_dir: &str) {
        prost_build::Config::new().type_attribute(".", "#[derive(::jinkela::Classicalize)]").out_dir(out_dir).compile_protos(&self.sources, &self.includes).unwrap();
    }
}
