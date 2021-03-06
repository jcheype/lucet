use lucetc::bindings::Bindings;
use lucetc::load;
use parity_wasm::elements::Module;
use std::collections::HashMap;
use std::path::PathBuf;

fn load(name: &str) -> Module {
    let watfile = PathBuf::from(&format!("tests/wasm/{}.wat", name));
    load::read_module(&watfile).expect(&format!("loading module from {:?}", watfile))
}

fn test_bindings() -> Bindings {
    let imports: HashMap<String, String> = [
        ("icalltarget".into(), "icalltarget".into()), // icall_import
        ("inc".into(), "inc".into()),                 // import
        ("imp_0".into(), "imp_0".into()),             // import_many
        ("imp_1".into(), "imp_1".into()),             // import_many
        ("imp_2".into(), "imp_2".into()),             // import_many
        ("imp_3".into(), "imp_3".into()),             // import_many
    ]
    .iter()
    .cloned()
    .collect();

    Bindings::env(imports)
}

mod programs {
    /// Tests of the `Program` datastructure.
    use super::load;
    use lucetc::bindings::Bindings;
    use lucetc::program::{table::TableElem, HeapSettings, Program};
    use parity_wasm::elements::ValueType;
    use std::path::PathBuf;

    #[test]
    fn fibonacci() {
        let m = load("fibonacci");
        let b = super::test_bindings();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));

        assert_eq!(p.import_functions().len(), 0);
        assert_eq!(p.globals().len(), 0);
        assert_eq!(p.defined_functions().len(), 1);
        assert_eq!(
            p.defined_functions().get(0).unwrap().symbol(),
            "guest_func_main"
        );
    }

    #[test]
    fn arith() {
        let m = load("arith");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));

        assert_eq!(p.import_functions().len(), 0);
        assert_eq!(p.globals().len(), 0);
        assert_eq!(p.defined_functions().len(), 1);
        assert_eq!(
            p.defined_functions().get(0).unwrap().symbol(),
            "guest_func_main"
        );
    }

    #[test]
    fn icall_import() {
        let m = load("icall_import");
        let b = Bindings::from_file(&PathBuf::from(
            "tests/bindings/icall_import_test_bindings.json",
        ))
        .unwrap();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));

        assert_eq!(p.import_functions().len(), 1);
        assert_eq!(p.import_functions()[0].module(), "env");
        assert_eq!(p.import_functions()[0].field(), "icalltarget");
        assert_eq!(p.globals().len(), 0);
        assert_eq!(p.defined_functions().len(), 4);
        assert_eq!(
            p.defined_functions().get(0).unwrap().symbol(),
            "guest_func_launchpad"
        );
        assert_eq!(
            p.get_table(0).unwrap().elements().get(0),
            Some(&TableElem::FunctionIx(2))
        ); // righttype1
        assert_eq!(
            p.get_table(0).unwrap().elements().get(1),
            Some(&TableElem::FunctionIx(3))
        ); // righttype2
        assert_eq!(
            p.get_table(0).unwrap().elements().get(2),
            Some(&TableElem::FunctionIx(4))
        ); // wrongtype
        assert_eq!(
            p.get_table(0).unwrap().elements().get(3),
            Some(&TableElem::FunctionIx(0))
        ); // righttype_imported
        assert_eq!(p.get_table(0).unwrap().elements().get(4), None);
    }

    #[test]
    fn icall() {
        let m = load("icall");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));

        assert_eq!(
            p.get_table(0).unwrap().elements().get(0),
            Some(&TableElem::FunctionIx(1))
        ); // righttype1
        assert_eq!(
            p.get_table(0).unwrap().elements().get(1),
            Some(&TableElem::FunctionIx(2))
        ); // righttype2
        assert_eq!(
            p.get_table(0).unwrap().elements().get(2),
            Some(&TableElem::FunctionIx(3))
        ); // wrongtype
        assert_eq!(p.get_table(0).unwrap().elements().get(4), None);
    }

    #[test]
    fn icall_sparse() {
        let m = load("icall_sparse");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));

        assert_eq!(
            p.get_table(0).unwrap().elements().get(0),
            Some(&TableElem::Empty)
        );
        assert_eq!(
            p.get_table(0).unwrap().elements().get(1),
            Some(&TableElem::FunctionIx(1))
        ); // righttype1
        assert_eq!(
            p.get_table(0).unwrap().elements().get(2),
            Some(&TableElem::FunctionIx(2))
        ); // righttype2
        assert_eq!(
            p.get_table(0).unwrap().elements().get(3),
            Some(&TableElem::FunctionIx(3))
        ); // wrongtype
        assert_eq!(
            p.get_table(0).unwrap().elements().get(4),
            Some(&TableElem::Empty)
        );
        assert_eq!(
            p.get_table(0).unwrap().elements().get(5),
            Some(&TableElem::Empty)
        );
        assert_eq!(p.get_table(0).unwrap().elements().get(6), None);
    }

    #[test]
    fn globals_import() {
        let m = load("globals_import");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));
        assert_eq!(p.globals().len(), 1);
        let g = p.globals()[0].as_import().expect("global is an import");
        assert_eq!(g.module(), "env");
        assert_eq!(g.field(), "x");
        assert_eq!(g.global_type.content_type(), ValueType::I32);
    }

    #[test]
    fn heap_spec_import() {
        use lucetc::program::memory::HeapSpec;
        let m = load("heap_spec_import");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));
        assert_eq!(
            p.heap_spec(),
            HeapSpec {
                // reserved and guard is liblucet_runtime_c standard
                reserved_size: 4 * 1024 * 1024,
                guard_size: 4 * 1024 * 1024,
                // initial size of import specified as 6 wasm pages
                initial_size: 6 * 64 * 1024,
                // max size of import is specified as 10 wasm pages
                max_size: Some(10 * 64 * 1024),
            }
        );
    }

    #[test]
    fn heap_spec_definition() {
        use lucetc::program::memory::HeapSpec;
        let m = load("heap_spec_definition");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));
        assert_eq!(
            p.heap_spec(),
            HeapSpec {
                // reserved and guard is liblucet_runtime_c standard
                reserved_size: 4 * 1024 * 1024,
                guard_size: 4 * 1024 * 1024,
                // initial size defined as 5 wasm pages
                initial_size: 5 * 64 * 1024,
                // no max size defined
                max_size: None,
            }
        );
    }

    #[test]
    fn heap_spec_none() {
        use lucetc::program::memory::HeapSpec;
        let m = load("heap_spec_none");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));
        assert_eq!(
            p.heap_spec(),
            HeapSpec {
                reserved_size: 0,
                guard_size: 0,
                initial_size: 0,
                max_size: None,
            }
        );
    }

    #[test]
    fn oversize_data_segment() {
        let m = load("oversize_data_segment");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect("instantiating is ok");
        assert!(
            p.data_initializers().is_err(),
            "data_initializers method returns error because data initializers are oversized"
        );
    }

    // XXX adding more negative tests like the one above is valuable - lets do it

    use lucetc::error::LucetcErrorKind;
    #[test]
    fn invalid_module() {
        // I used the `wast2json` tool to produce the file invalid.wasm from an assert_invalid part
        // of a spectest (call.wast)
        let wasmfile = PathBuf::from("tests/wasm/invalid.wasm");
        let m = load::read_module(&wasmfile).expect(&format!("loading module from {:?}", wasmfile));
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h);
        assert!(p.is_err());
        assert_eq!(*p.err().unwrap().get_context(), LucetcErrorKind::Validation);
    }

    #[test]
    fn start_section() {
        let m = load("start_section");
        let b = Bindings::empty();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("instantiating program"));
        assert!(
            p.module().start_section().is_some(),
            "start section is defined"
        );
    }
}

mod compile {
    // Tests for compilation completion
    use super::load;
    use lucetc::compile;
    use lucetc::compiler::OptLevel;
    use lucetc::program::{HeapSettings, Program};
    fn run_compile_test(file: &str) {
        let m = load(file);
        let b = super::test_bindings();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("make program for {}", file));
        compile(&p, file.into(), OptLevel::Best).expect(&format!("compile {}", file));
    }
    macro_rules! compile_test {
        ($base_name:ident) => {
            #[test]
            fn $base_name() {
                run_compile_test(stringify!($base_name))
            }
        };
    }

    compile_test!(arith);
    compile_test!(call);
    compile_test!(data_segment);
    compile_test!(fibonacci);
    compile_test!(globals_definition);
    compile_test!(globals_import);
    compile_test!(icall);
    compile_test!(icall_import);
    compile_test!(icall_sparse);
    compile_test!(import);
    compile_test!(import_many);
    compile_test!(locals);
    compile_test!(locals_csr);
    compile_test!(memory);
    compile_test!(return_at_end);
    compile_test!(current_memory);
    compile_test!(grow_memory);
    compile_test!(unreachable_code);
    compile_test!(start_section);
}

mod execute {
    // Tests for compilation correctness
    use super::load;
    use lucetc::compile;
    use lucetc::compiler::OptLevel;
    use lucetc::program::{HeapSettings, Program};
    use std::process::Command;
    use std::str;

    fn run_execute_test(file: &str) {
        // Compile the wasm file
        let m = load(file);
        let b = super::test_bindings();
        let h = HeapSettings::default();
        let p = Program::new(m, b, h).expect(&format!("make program for {}", file));
        let comp = compile(&p, file, OptLevel::Best).expect(&format!("compile test for {}", file));

        // Set up output fils
        let tmp_dir = tempfile::Builder::new()
            .prefix("execute_test_out")
            .tempdir()
            .expect("Failed to create temp dir");
        let obj_path = tmp_dir.path().join(format!("{}.o", file));
        let obj_file = obj_path
            .to_str()
            .expect("failed to convert obj_file pathbuf to str");

        let exec_path = tmp_dir.path().join(format!("{}_test", file));
        let exec_file = exec_path
            .to_str()
            .expect("failed to convert exec_file pathbuf to str");

        // Write the compiled object
        let obj = comp.codegen().expect("generate code");
        obj.write(&obj_path).expect("write object file");

        // Mark the wasm table as global, so we can inspect it in the tests.
        let objcopy = Command::new("objcopy")
            .arg("--globalize-symbol=guest_table_0")
            .arg(obj_file)
            .output()
            .expect("failed to execute objcopy");
        if !objcopy.status.success() {
            let stdout = str::from_utf8(&objcopy.stdout).unwrap();
            let stderr = str::from_utf8(&objcopy.stderr).unwrap();
            println!(
                "objcopy status: {}",
                objcopy.status.code().expect("status code error")
            );
            println!("objcopy stdout: {}", stdout);
            println!("objcopy stderr: {}", stderr);
        }
        assert!(objcopy.status.success(), "objcopy failed!");

        // Invoke GCC to compile and link the harness and the object.
        let harness_file = format!("tests/harnesses/{}.c", file);
        let output = Command::new("gcc")
            .arg("--std=c99")
            .arg(harness_file)
            .arg("tests/harnesses/vm.c")
            .arg("tests/harnesses/globals.c")
            .arg(obj_file)
            .args(&["-o", exec_file])
            .output()
            .expect("failed to execute GCC");

        if !output.status.success() {
            let stdout = str::from_utf8(&output.stdout).unwrap();
            let stderr = str::from_utf8(&output.stderr).unwrap();
            println!(
                "GCC status: {}",
                output.status.code().expect("status code error")
            );
            println!("GCC stdout: {}", stdout);
            println!("GCC stderr: {}", stderr);
        }
        assert!(output.status.success(), "GCC failed!");

        // Finally, invoke the test itself.
        let output = Command::new(exec_file)
            .output()
            .expect("failed to run execute test");

        if !output.status.success() {
            match output.status.code() {
                Some(code) => println!("Test status: {}", code),
                None => println!("Test terminated by signal"),
            };
            let stdout = str::from_utf8(&output.stdout).unwrap();
            let stderr = str::from_utf8(&output.stderr).unwrap();
            println!("Test stdout: {}", stdout);
            println!("Test stderr: {}", stderr);
        }
        assert!(output.status.success(), "Execute test failed!");
    }

    macro_rules! execute_test {
        ($base_name:ident) => {
            #[test]
            pub fn $base_name() {
                run_execute_test(stringify!($base_name));
            }
        };
    }

    execute_test!(fibonacci);
    execute_test!(call);
    execute_test!(icall);
    execute_test!(locals);
    execute_test!(locals_csr);
    execute_test!(data_segment);
    execute_test!(import_many);
    execute_test!(icall_import);
    execute_test!(icall_sparse);
    execute_test!(current_memory);
    execute_test!(grow_memory);
    execute_test!(heap_spec_import);
    execute_test!(heap_spec_definition);
    execute_test!(globals_definition);
}
