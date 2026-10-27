mod shell;


fn main() -> rustyline::Result<()> {
    shell::session::run()
}