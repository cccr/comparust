use ggez::graphics::FillOptions;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, Text, TextFragment, Rect};
use rfd::FileDialog;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

struct FolderDiffApp {
    folder1: Option<PathBuf>,
    folder2: Option<PathBuf>,
    diff_output: Vec<String>, // Change to Vec<String> for individual lines
    progress: f32,
    folder1_sender: mpsc::Sender<Option<PathBuf>>,
    folder2_sender: mpsc::Sender<Option<PathBuf>>,
    folder1_receiver: mpsc::Receiver<Option<PathBuf>>,
    folder2_receiver: mpsc::Receiver<Option<PathBuf>>,
    recursive: bool,
    ignore_ds_store: bool,
    display_folder1_only: bool,
    display_folder2_only: bool,
}

impl FolderDiffApp {
    fn new(_ctx: &mut Context) -> FolderDiffApp {
        let (folder1_tx, folder1_rx) = mpsc::channel();
        let (folder2_tx, folder2_rx) = mpsc::channel();

        FolderDiffApp {
            folder1: None,
            folder2: None,
            diff_output: vec!["Select two folders to compare.\nPress Enter to run diff.".to_string()],
            progress: 0.0,
            folder1_sender: folder1_tx,
            folder2_sender: folder2_tx,
            folder1_receiver: folder1_rx,
            folder2_receiver: folder2_rx,
            recursive: true,
            ignore_ds_store: true,
            display_folder1_only: false,
            display_folder2_only: false,
        }
    }

    fn run_diff(&mut self) {
        if let (Some(ref folder1), Some(ref folder2)) = (&self.folder1, &self.folder2) {
            let mut args = vec!["-rq"];
            if self.recursive {
                args.push("-r");
            }
            args.extend_from_slice(&[folder1.to_str().unwrap(), folder2.to_str().unwrap()]);

            self.progress = 0.0;
            let output = Command::new("diff")
                .args(&args)
                .output();

            match output {
                Ok(output) => {
                    let output_text = String::from_utf8_lossy(&output.stdout).to_string();
                    let filtered_output: Vec<String> = if self.ignore_ds_store {
                        output_text
                            .lines()
                            .filter(|line| !line.contains(".DS_Store"))
                            .map(String::from)
                            .collect()
                    } else {
                        output_text.lines().map(String::from).collect()
                    };

                    self.diff_output = filtered_output;
                }
                Err(err) => {
                    self.diff_output = vec![format!("Error running diff command: {}", err)];
                }
            }
        } else {
            self.diff_output = vec!["Both folder paths are required.".to_string()];
        }
    }

    fn open_folder_dialog(&self, is_first_folder: bool) {
        let sender = if is_first_folder {
            self.folder1_sender.clone()
        } else {
            self.folder2_sender.clone()
        };

        thread::spawn(move || {
            let folder_path = FileDialog::new().pick_folder();
            sender.send(folder_path).expect("Failed to send folder path");
        });
    }

    fn export_results(&self) {
        if !self.diff_output.is_empty() {
            let mut file = File::create("diff_results.txt").expect("Unable to create file");
            file.write_all(self.diff_output.join("\n").as_bytes())
                .expect("Unable to write data");
        }
    }

    fn clear_selections(&mut self) {
        self.folder1 = None;
        self.folder2 = None;
        self.diff_output.clear();
    }

    fn filter_and_color_diff_output(&self) -> Vec<TextFragment> {
        self.diff_output.iter()
            .filter_map(|line| {
                if self.display_folder1_only && line.contains("Only in") && line.contains(self.folder1.as_ref()?.to_str()?) {
                    Some(TextFragment::new(line.as_str()).color(Color::BLUE))
                } else if self.display_folder2_only && line.contains("Only in") && line.contains(self.folder2.as_ref()?.to_str()?) {
                    Some(TextFragment::new(line.as_str()).color(Color::GREEN))
                } else if !self.display_folder1_only && !self.display_folder2_only {
                    if line.contains(self.folder1.as_ref()?.to_str()?) {
                        Some(TextFragment::new(line.as_str()).color(Color::BLUE))
                    } else if line.contains(self.folder2.as_ref()?.to_str()?) {
                        Some(TextFragment::new(line.as_str()).color(Color::GREEN))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn swap_folders(&mut self) {
        std::mem::swap(&mut self.folder1, &mut self.folder2);
    }
}

impl EventHandler for FolderDiffApp {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if let Ok(Some(folder_path)) = self.folder1_receiver.try_recv() {
            self.folder1 = Some(folder_path);
        }
        if let Ok(Some(folder_path)) = self.folder2_receiver.try_recv() {
            self.folder2 = Some(folder_path);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let folder1_display = self.folder1.as_ref().map_or("None".to_string(), |p| p.display().to_string());
        let folder2_display = self.folder2.as_ref().map_or("None".to_string(), |p| p.display().to_string());

        let instruction_text = Text::new(format!(
            "Folder 1: {}\nFolder 2: {}\n\nOptions:\n- Recursive (R): {}\n- Ignore .DS_Store (D): {}\n- Show Only Folder 1 (1): {}\n- Show Only Folder 2 (2): {}\n\nActions:\n- Swap Folders (S)\n- Clear Selections (C)\n- Export Results (E)\n\nPress Enter to run diff.",
            folder1_display, folder2_display, self.recursive, self.ignore_ds_store, self.display_folder1_only, self.display_folder2_only
        ));
        graphics::draw(ctx, &instruction_text, (ggez::mint::Point2 { x: 10.0, y: 10.0 },))?;

        // Draw separator
        let separator_rect = Rect::new(10.0, instruction_text.height(ctx) + 20.0, 300.0, 2.0);
        let colored_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::Fill(FillOptions::default()), separator_rect, graphics::Color::WHITE)?;
        graphics::draw(ctx, &colored_mesh, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;

        // Draw results
        let result_y_start = instruction_text.height(ctx) + 30.0;
        let result_height = 400.0;  // Height for the results area

        let mut current_y = result_y_start;

        let results = self.filter_and_color_diff_output();
        for line in results {
            let result_text = Text::new(line);
            if current_y < result_y_start + result_height {
                graphics::draw(ctx, &result_text, (ggez::mint::Point2 { x: 10.0, y: current_y },))?;
                current_y += result_text.height(ctx);
            }
        }

        graphics::present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Return => self.run_diff(),
            KeyCode::F1 => self.open_folder_dialog(true),
            KeyCode::F2 => self.open_folder_dialog(false),
            KeyCode::R => self.recursive = !self.recursive,
            KeyCode::D => self.ignore_ds_store = !self.ignore_ds_store,
            KeyCode::Key1 => self.display_folder1_only = !self.display_folder1_only,
            KeyCode::Key2 => self.display_folder2_only = !self.display_folder2_only,
            KeyCode::S => self.swap_folders(),
            KeyCode::C => self.clear_selections(),
            KeyCode::E => self.export_results(),
            _ => {}
        }
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Folder Diff", "klmv.dev")
        .build()
        .expect("Failed to build ggez context");

    let app = FolderDiffApp::new(&mut ctx);

    ggez::event::run(ctx, event_loop, app)
}