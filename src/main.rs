use dupelib::Opt;
use structopt::StructOpt;

extern crate dupelib;

// fn main() {

// }

extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::FileChooserDialog;
use gtk::{
    Application, ApplicationWindow, Button, FileChooserAction, ResponseType, Window, WindowType,
};
use std::path::PathBuf;

pub struct OpenDialog(FileChooserDialog);
impl OpenDialog {
    pub fn new() -> OpenDialog {
        // Create a new file chooser dialog for opening a file.
        let open_dialog = FileChooserDialog::new(
            Some("Open"),
            Some(&Window::new(WindowType::Popup)),
            FileChooserAction::SelectFolder,
        );

        // Add the cancel and open buttons to that dialog.
        open_dialog.add_button("Cancel", ResponseType::Cancel.into());
        open_dialog.add_button("Open", ResponseType::Ok.into());
        open_dialog.set_select_multiple(true);

        OpenDialog(open_dialog)
    }

    pub fn run(&self) -> Vec<PathBuf> {
        let open_dialog = &self.0;
        if open_dialog.run() == ResponseType::Ok.into() {
            open_dialog.get_filenames()
        } else {
            Vec::<_>::new()
        }
    }
}

impl Drop for OpenDialog {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

fn main() {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Deduper");
        window.set_default_size(600, 150);

        let button = Button::new_with_label("Choose folder");
        button.connect_clicked(|_| {
            let open_dialog = OpenDialog::new();
            let files = open_dialog.run();
            let options = Opt {
            	paths: files,
			    timing: false,
			    debug: false,
			    output: None,
			    minimum: None,
			    sort: true,
            };
		    let dupe_count = dupelib::detect_dupes(options);
		    println!("{} dupes detected", dupe_count);
        });
        window.add(&button);

        window.show_all();
    });

    application.run(&[]);
}
