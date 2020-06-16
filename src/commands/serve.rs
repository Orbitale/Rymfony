use clap::App;
use clap::Arg;

pub fn cmd<'a, 'b>() -> App<'a, 'b> {
    let app = App::new("test");
    app
        .about("does testing things")
        .arg(Arg::new("list").short('l').about("lists test values"))
    ;
    return app;
}
