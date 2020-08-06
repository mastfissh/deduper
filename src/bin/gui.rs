#![windows_subsystem = "windows"]

extern crate crossbeam_channel;
extern crate dupelib;

use dupelib::detect_dupes;
use dupelib::Opt;

use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::unbounded;
use crossbeam_channel::RecvError;

use druid::commands;
use druid::platform_menus;
use druid::widget::{Button, Either, Flex, Label, List, Scroll};
use druid::{
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink,
    FileDialogOptions, FileInfo, Lens, LocalizedString, MenuDesc, MenuItem, Selector, SysMods,
    Target, UnitPoint, Widget, WidgetExt, WindowDesc,
};

pub struct Delegate {
    eventsink: ExtEventSink,
}

#[derive(Clone, Data, Default, Lens, Debug)]
pub struct AppState {
    pub paths: Arc<Vec<DisplayablePath>>,
    pub dupes: Arc<Vec<String>>,
    pub processing: bool,
    pub progress_info: String,
}

#[derive(Clone, Data)]
pub struct DisplayablePath {
    #[data(same_fn = "PartialEq::eq")]
    pathbuf: PathBuf,
}

pub const START_DUPE: Selector = Selector::new("start_dupe");

pub const FINISH_DUPE: Selector = Selector::new("finish_dupe");

pub const PROGRESS_UPDATE: Selector = Selector::new("progress_update");

impl Display for DisplayablePath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.pathbuf.as_path().to_string_lossy())
    }
}

impl Debug for DisplayablePath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.pathbuf.as_path().to_string_lossy())
    }
}

fn run_dupe_detect(sink: ExtEventSink, options: Opt) {
    let sinkin = sink.clone();
    let (sender, receiver) = unbounded();
    thread::spawn(move || {
        let dupes = detect_dupes(options, Some(sender));
        sinkin
            .submit_command(FINISH_DUPE, dupes, None)
            .expect("command failed to submit");
    });

    thread::spawn(move || {
        let mut cont = true;
        while cont {
            cont = false;
            let data = receiver.recv();
            if data != Err(RecvError) {
                cont = true;
                sink.submit_command(PROGRESS_UPDATE, data.unwrap().to_string(), None)
                    .expect("command failed to submit");
            }
        }
    });
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: &Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> bool {
        match cmd.selector {
            druid::commands::OPEN_FILE => {
                let info = cmd.get_object::<FileInfo>().expect("api violation");
                let pathbuf = info.path().clone().to_path_buf();
                Arc::make_mut(&mut data.paths).push(DisplayablePath { pathbuf });
                true
            }
            START_DUPE => {
                data.processing = true;
                data.dupes = Arc::new(Vec::<String>::new());
                let paths = data.paths.iter().map(|path| path.pathbuf.clone()).collect();
                let options = Opt::from_paths(paths);
                run_dupe_detect(self.eventsink.clone(), options);
                true
            }
            FINISH_DUPE => {
                data.processing = false;
                let dupes = cmd.get_object::<Vec<String>>().expect("api violation");
                data.dupes = Arc::new(dupes.to_vec());

                true
            }
            PROGRESS_UPDATE => {
                let update = cmd.get_object::<String>().expect("api violation");
                data.progress_info = update.to_string();

                true
            }
            _ => true,
        }
    }
}

fn ui_builder() -> impl Widget<AppState> {
    let button = Button::new("Check")
        .on_click(|ctx, _data: &mut AppState, _env| {
            let cmd = Command::new(START_DUPE, 0);
            ctx.submit_command(cmd, None);
        })
        .padding(5.0);
    let button_placeholder =
        Label::new(|data: &AppState, _env: &Env| format!("{}", data.progress_info))
            .padding(5.0)
            .center();

    let either = Either::new(|data, _env| data.processing, button_placeholder, button);

    let paths_to_check = Label::new(LocalizedString::new("Paths to check")).padding(5.0);

    let discovered_dupes = Label::new(LocalizedString::new("Discovered Dupes")).padding(5.0);

    Flex::column()
        .with_child(paths_to_check)
        .with_flex_child(
            Scroll::new(List::new(|| {
                Label::new(|item: &DisplayablePath, _env: &_| format!("#{}", item))
                    .align_vertical(UnitPoint::LEFT)
                    .padding(10.0)
                    .expand()
                    .height(50.0)
                    .background(Color::rgb(0.5, 0.5, 0.5))
            }))
            .vertical()
            .lens(AppState::paths),
            1.0,
        )
        .with_child(either)
        .with_child(discovered_dupes)
        .with_flex_child(
            Scroll::new(List::new(|| {
                Label::new(|item: &String, _env: &_| format!("#{}", item))
                    .align_vertical(UnitPoint::LEFT)
                    .padding(10.0)
                    .expand()
                    .height(50.0)
                    .background(Color::rgb(0.5, 0.5, 0.5))
            }))
            .vertical()
            .lens(AppState::dupes),
            1.0,
        )
}
fn main() {
    let main_window = WindowDesc::new(|| ui_builder())
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu())
        .window_size((700.0, 500.0));
    let app = AppLauncher::with_window(main_window);

    let delegate = Delegate {
        eventsink: app.get_external_handle(),
    };
    app.delegate(delegate)
        .use_simple_logger()
        .launch(AppState::default())
        .expect("launch failed");
}

fn make_menu() -> MenuDesc<AppState> {
    let menu = MenuDesc::empty();
    menu.append(file_menu())
}

fn file_menu() -> MenuDesc<AppState> {
    MenuDesc::new(LocalizedString::new("common-menu-file-menu"))
        .append(platform_menus::mac::file::new_file().disabled())
        .append(
            MenuItem::new(
                LocalizedString::new("common-menu-file-open"),
                Command::new(
                    commands::SHOW_OPEN_PANEL,
                    FileDialogOptions::new().select_directories(),
                ),
            )
            .hotkey(SysMods::Cmd, "o"),
        )
        .append_separator()
        .append(platform_menus::mac::file::close())
}
