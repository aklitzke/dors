use clap::App;

fn main() {
    std::process::exit({
        let app_matches = dors::set_app_options(
            App::new("dors -- do things, for rust!")
                .bin_name("dors")
                .max_term_width(80),
        )
        .get_matches();

        dors::process_cmd(&app_matches)
    });
}
