use glib::clone;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Button;
use gtk4::Grid;
use gtk4::Label;
use gtk4::ScrolledWindow;
use gtk4::TextView;
use gtk4::{FileChooserAction, FileChooserNative, ResponseType};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("dev.klmn.comparust")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Folder Comparison Tool")
            .default_width(600)
            .default_height(400)
            .build();

        let folder1_label = Label::new(Some("Folder 1: Not Selected"));
        let folder2_label = Label::new(Some("Folder 2: Not Selected"));

        let choose_folder1_button = Button::with_label("Choose Folder 1");
        let choose_folder2_button = Button::with_label("Choose Folder 2");
        let compare_button = Button::with_label("Compare Folders");
        let export_button = Button::with_label("Export Results");

        let results_view = TextView::new();
        results_view.set_editable(false);

        let scrolled_window = ScrolledWindow::builder()
            .child(&results_view)
            .vexpand(true)
            .build();

        let grid = Grid::builder()
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .row_spacing(5)
            .column_spacing(5)
            .build();

        grid.attach(&folder1_label, 0, 0, 3, 1);
        grid.attach(&folder2_label, 0, 1, 3, 1);
        grid.attach(&choose_folder1_button, 3, 0, 1, 1);
        grid.attach(&choose_folder2_button, 3, 1, 1, 1);
        grid.attach(&compare_button, 0, 2, 4, 1);
        grid.attach(&scrolled_window, 0, 3, 4, 1);
        grid.attach(&export_button, 0, 4, 4, 1);

        let results_buffer = results_view.buffer();

        let folder1 = Rc::new(RefCell::new(None));
        let folder2 = Rc::new(RefCell::new(None));

        choose_folder1_button.connect_clicked(clone!(@weak folder1_label, @weak window, @strong folder1 => move |_| {
            create_file_chooser_dialog(&window, "Select Folder 1", FileChooserAction::SelectFolder, {
                let folder1 = folder1.clone();
                move |folder_path| {
                    if let Some(path) = folder_path {
                        folder1.replace(Some(path.clone())); // Clone the path before replacing it in folder1.
                        folder1_label.set_text(&format!("Folder 1: {:?}", path)); // Use the original path here.
                    }
                }
            });
        }));

        choose_folder2_button.connect_clicked(clone!(@weak folder2_label, @weak window, @strong folder2 => move |_| {
            create_file_chooser_dialog(&window, "Select Folder 2", FileChooserAction::SelectFolder, {
                let folder2 = folder2.clone();
                move |folder_path| {
                    if let Some(path) = folder_path {
                        folder2.replace(Some(path.clone())); // Update the value in Rc<RefCell<>>.
                        folder2_label.set_text(&format!("Folder 2: {:?}", path));
                    }
                }
            });
        }));

        compare_button.connect_clicked(clone!(@weak results_buffer => move |_| {
            if let (Some(ref path1), Some(ref path2)) = (folder1.borrow().as_ref(), folder2.borrow().as_ref()) {
                let (only_in_folder1, only_in_folder2) = compare_folders(path1, path2);

                let mut results_text = String::new();
                results_text.push_str("Files only in Folder 1:\n");
                for file in &only_in_folder1 {
                    results_text.push_str(&format!("{}\n", file.display()));
                }
                results_text.push_str("\nFiles only in Folder 2:\n");
                for file in &only_in_folder2 {
                    results_text.push_str(&format!("{}\n", file.display()));
                }

                results_buffer.set_text(&results_text);
            } else {
                results_buffer.set_text("Please select both folders before comparing.");
            }
        }));

        export_button.connect_clicked(clone!(@weak results_buffer, @weak window => move |_| {
            create_file_chooser_dialog(&window, "Select Export Location", FileChooserAction::Save, move |export_path| {
                if let Some(file) = export_path {
                    if let Ok(mut file) = std::fs::File::create(file) {
                        // Get the text from the results buffer
                        let export_text = results_buffer.text(&results_buffer.start_iter(), &results_buffer.end_iter(), false);
                        // Write the text to the file
                        if let Err(e) = std::io::Write::write_all(&mut file, export_text.as_bytes()) {
                            eprintln!("Failed to write to file: {}", e);
                        }
                    } else {
                        eprintln!("Failed to create file at the specified location.");
                    }
                }
            });
        }));


        window.set_child(Some(&grid));
        window.show();
    });

    app.run();
}

fn create_file_chooser_dialog<W: IsA<gtk4::Window>>(
    window: &W,
    title: &str,
    chooser_action: FileChooserAction,
    callback: impl Fn(Option<PathBuf>) + 'static,
) {
    // Create a new file chooser dialog for folder selection

    let native = FileChooserNative::new(
        Some(title),
        Some(window),
        chooser_action,
        Some("Select"),
        Some("Cancel"),
    );
    let dialog = native;

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            // If a folder is selected, pass the path to the callback
            let folder_path = dialog.file().and_then(|folder| folder.path());
            callback(folder_path);
        } else {
            callback(None);
        }
        dialog.destroy(); // Close the dialog
    });

    dialog.show();
}

fn get_files_in_folder(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .map(|read_dir| {
            read_dir
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_else(|_| Vec::new())
}

fn compare_folders(folder1: &Path, folder2: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let files1: HashSet<PathBuf> = get_files_in_folder(folder1).into_iter().collect();
    let files2: HashSet<PathBuf> = get_files_in_folder(folder2).into_iter().collect();

    let only_in_folder1: Vec<PathBuf> = files1.difference(&files2).cloned().collect();
    let only_in_folder2: Vec<PathBuf> = files2.difference(&files1).cloned().collect();

    (only_in_folder1, only_in_folder2)
}
