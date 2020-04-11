#![windows_subsystem = "windows"]

extern crate dupelib;

use druid::commands;
use druid::platform_menus;
use druid::ExtEventSink;
use druid::Selector;
use dupelib::detect_dupes;
use dupelib::Opt;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::thread;


use druid::AppDelegate;
use druid::DelegateCtx;
use druid::Target;

use druid::{AppLauncher, Widget, WindowDesc};
use druid::{
    Command, Data, Env, FileDialogOptions, FileInfo, Lens, LocalizedString, MenuDesc, MenuItem,
    SysMods,
};
#[derive(Default)]
pub struct Delegate {
    eventsink: Option<ExtEventSink>,
}

use druid::widget::{Button, Flex, Label};
use druid::WidgetExt;

use std::path::PathBuf;
use std::sync::Arc;

use druid::widget::{List, Scroll};
use druid::{Color, UnitPoint};

#[derive(Clone, Data, Default, Lens, Debug)]
pub struct AppState {
    pub paths: Arc<Vec<DisplayablePath>>,
    pub dupes: Arc<Vec<String>>,
}

#[derive(Clone, Data)]
pub struct DisplayablePath {
    #[data(same_fn = "PartialEq::eq")]
    pathbuf: PathBuf,
}

pub const START_DUPE: Selector = Selector::new("start_dupe");

pub const FINISH_DUPE: Selector = Selector::new("finish_dupe");

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
    thread::spawn(move || {
        let dupes = detect_dupes(options);
        sink.submit_command(FINISH_DUPE, dupes, None).expect("command failed to submit");
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
                let paths = data.paths.iter().map(|path| path.pathbuf.clone()).collect();
                let options = Opt::from_paths(paths);
                run_dupe_detect(self.eventsink.as_ref().unwrap().clone(), options);
                true
            }
            FINISH_DUPE => {
                let dupes = cmd.get_object::<Vec<String>>().expect("api violation");
                data.dupes = Arc::new(dupes.to_vec());
                // dbg!(dupes);
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

    Flex::column()
        .with_flex_child(
            Scroll::new(List::new(|| {
                Label::new(|item: &DisplayablePath, _env: &_| format!("List item #{}", item))
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
        .with_child(button)
        .with_flex_child(
            Scroll::new(List::new(|| {
                Label::new(|item: &String, _env: &_| format!("List item #{}", item))
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
    let mut delegate = Delegate::default();
    let main_window = WindowDesc::new(|| ui_builder())
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu());
    let app = AppLauncher::with_window(main_window);
    delegate.eventsink = Some(app.get_external_handle());

    app.delegate(delegate)
        .use_simple_logger()
        .launch( AppState::default())
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
