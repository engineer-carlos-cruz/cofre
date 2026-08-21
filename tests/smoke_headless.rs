mod test_startup {
    use std::process::{Command, Stdio};

    fn ejecutar_headless() -> (Option<i32>, String) {
        let salida = Command::new(env!("CARGO_BIN_EXE_cofre"))
            .stdout(Stdio::piped())
            .output()
            .expect("el binario cofre debería poder ejecutarse");
        (
            salida.status.code(),
            String::from_utf8_lossy(&salida.stderr).into_owned(),
        )
    }

    #[test]
    fn stdout_no_tty_es_error() {
        let (codigo, stderr) = ejecutar_headless();
        assert!(stderr.contains("cofre:"), "stderr: {stderr}");
        assert_ne!(codigo, Some(0));
    }

    #[test]
    fn exit_distinto_de_cero() {
        let (codigo, _) = ejecutar_headless();
        assert_ne!(codigo, Some(0));
        assert_ne!(codigo, None);
    }

    #[test]
    fn mensaje_via_stderr_legible() {
        let (_, stderr) = ejecutar_headless();
        assert!(
            stderr.contains("no es una terminal interactiva"),
            "stderr: {stderr}"
        );
    }
}
