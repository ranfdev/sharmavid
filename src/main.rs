#[rustfmt::skip]
mod config;
mod glib_utils;
mod invidious;
mod widgets;

use gettextrs::{gettext, LocaleCategory};
use gtk::{gdk, gio, glib};
pub use invidious::client::Client;
use libadwaita as adw;
use libadwaita::prelude::*;
use widgets::SharMaVidWindow;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("SharMaVid"));

    gtk::init().expect("Unable to start GTK4");
    adw::init();

    widgets::Thumbnail::static_type();

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);
    let theme = gtk::IconTheme::for_display(&gdk::Display::default().unwrap()).unwrap();
    theme.add_resource_path("/com/ranfdev/SharMaVid/icons/");

    let app = adw::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE);

    let client = Client::new("https://inv.riverside.rocks".to_string())
        .expect("failed creating backend client");
    app.connect_activate(move |app| {
        let win = SharMaVidWindow::new(&app, client.clone());
        win.load_popular();
        win.present();
    });

    app.run();
}
