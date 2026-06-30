use std::fs;
use std::fs::File;
use eframe::{Frame};
use egui::{Context, TextureOptions, Vec2};
use crate::cpu::cpu::CPUState;
use crate::emulator::Emulator;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct UiApp {
    #[serde(skip)]
    emulator: Emulator,

    #[serde(skip)]
    memory_inspect_target: String,

    #[serde(skip)]
    memory_inspect_value: String,

    #[serde(skip)]
    rom_filepath: String,

    #[serde(skip)]
    tracelogger_text: String,

    tracelogger_view: bool,
    pattern_table_view: bool,

    pattern_table: Vec<u8>
}

impl Default for UiApp {
    fn default() -> Self {
        Self {
            emulator: Emulator::default(),
            memory_inspect_target: String::default(),
            memory_inspect_value: String::default(),
            rom_filepath: String::default(),
            tracelogger_text: String::default(),
            tracelogger_view: false,
            pattern_table_view: false,
            pattern_table: Vec::with_capacity(256*128*3),
        }
    }
}

impl UiApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn load_pattern_table(&mut self) {
        self.pattern_table = vec![0; 256*128*3];
        for table in 0..2 {
            for row in 0..16 {
                for column in 0..16 {
                    for y in 0..8 {
                        let low_byte = self.emulator.chrdata[y + column*16 + row*256 + table*4096];
                        let high_byte = self.emulator.chrdata[8 + y + column*16 + row*256 + table*4096];
                        for x in 0..8 {
                            let mut two_bit = if ((low_byte >> (7 - x)) & 1) == 1 { 1 } else { 0 };
                            two_bit += if ((high_byte >> (7 - x)) & 1) == 1 { 1 } else { 0 };

                            self.pattern_table[(x + column*8 + table*128)*3 + (768 * (y + row*8))] = two_bit * 85;
                            self.pattern_table[(x + column*8 + table*128)*3 + 1 + (768 * (y + row*8))] = two_bit * 85;
                            self.pattern_table[(x + column*8 + table*128)*3 + 2 + (768 * (y + row*8))] = two_bit * 85;
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for UiApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.tracelogger_view {
            self.emulator.run_frame();
        } else {
            self.emulator.master_cycle(); // TODO: Split out to an always-called loop
        }
        ctx.request_repaint();
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Fairnes");

            ui.horizontal(|ui| {
                if ui.button("Load 1_Example.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/1_Example.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 2_ReadWrite.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/2_ReadWrite.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 3_Branches.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/3_Branches.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 4_TheStack.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/4_TheStack.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 5_instructions1.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/5_Instructions1.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 6_Instructions2.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/6_Instructions2.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                }

                if ui.button("Load 7_Graphics.nes").clicked() {
                    let bytes = include_bytes!("../__PatreonRoms/7_Graphics.nes");
                    self.emulator.load_cartridge(bytes.to_vec());
                    self.load_pattern_table();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Rom to load: ");
                ui.text_edit_singleline(&mut self.rom_filepath);
                if ui.button("Load").clicked() {
                    if fs::exists(&self.rom_filepath).expect("Unable to check if file exists") {
                        let bytes = fs::read(self.rom_filepath.clone()).unwrap();
                        self.emulator.load_cartridge(bytes);
                        self.load_pattern_table();
                    }
                }
            });



            ui.horizontal(|ui| {
                ui.label("Memory value to inspect: ");
                ui.text_edit_singleline(&mut self.memory_inspect_target);
                if ui.button("Probe").clicked() {
                    if let Ok(num) = self.memory_inspect_target.parse::<u16>() {
                        self.memory_inspect_value = format!("{:x}", self.emulator.mem_read(num));
                    } else {
                        self.memory_inspect_value = "Invalid number".to_string();
                    }
                }

                if self.memory_inspect_value.is_empty() == false {
                    ui.label(&self.memory_inspect_value);
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("A: {:x}", self.emulator.cpu.a));
                ui.label(format!("X: {:x}", self.emulator.cpu.x));
                ui.label(format!("Y: {:x}", self.emulator.cpu.y));
                ui.label(format!("PC: {:x}", self.emulator.cpu.pc));
            });

            ui.label(&self.emulator.last_text);

            ui.checkbox(&mut self.tracelogger_view, "View Tracelogger");
            ui.checkbox(&mut self.pattern_table_view, "View Pattern Tables");

            ui.separator();

            if self.pattern_table_view {
                let texture = egui::ColorImage::from_rgb([256,128], self.pattern_table.as_slice());
                let handle = ui.ctx().load_texture("pattern_table", texture, TextureOptions::default());
                let sized_texture = egui::load::SizedTexture::new(handle.id(), Vec2 {x: 256f32, y: 128f32});
                ui.image(sized_texture);
            }

            if self.tracelogger_view { // TODO: Fix tracelogger (instructions are one behind, don't have opcodes, and just need better formatting)
                if self.emulator.cpu.state == CPUState::NeedInstruction {
                    self.tracelogger_text += &format!("\n{:X}: {:X}  {} A: {:X}, X: {:X}, Y: {:X}, SP: {:X}, P: {:X}", self.emulator.cpu.pc,
                                                      self.emulator.cpu.instruction.opcode, self.emulator.cpu.instruction.name,self.emulator.cpu.a, self.emulator.cpu.x, self.emulator.cpu.y,
                                                        self.emulator.cpu.sp, self.emulator.cpu.p.get());
                }
                ui.text_edit_multiline(&mut self.tracelogger_text);
            }


            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}