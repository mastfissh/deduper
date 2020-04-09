extern crate dupelib;
use druid::commands;
use druid::platform_menus;
use dupelib::detect_dupes;
use dupelib::Opt;
use std::fmt;
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
#[derive(Debug, Default)]
pub struct Delegate;

use druid::widget::{Button, Flex, Label};
use druid::WidgetExt;

use std::path::PathBuf;
use std::sync::Arc;

use druid::widget::{List, Scroll};
use druid::{Color, UnitPoint};

#[derive(Clone, Data, Default, Lens)]
pub struct AppState {
    pub paths: Arc<Vec<DisplayablePath>>,
    pub dupes: Arc<Vec<String>>,
}

#[derive(Clone, Data)]
pub struct DisplayablePath {
    #[data(same_fn = "PartialEq::eq")]
    pathbuf: PathBuf,
}

impl Display for DisplayablePath {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.pathbuf.as_path().to_string_lossy())
    }
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
            _ => true,
        }
    }
}

fn ui_builder() -> impl Widget<AppState> {
    let button = Button::new("Check")
        .on_click(|ctx, data: &mut AppState, _env| {
            let paths = data.paths.iter().map(|path| path.pathbuf.clone()).collect();
            let options = Opt::from_paths(paths);
            
            thread::spawn(|| {
                let dupes = detect_dupes(options);
                // let cmd = Command::new();
                // ctx.submit_command(druid::commands::HIDE_APPLICATION, None);
                dbg!(dupes)
            });
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
    let main_window = WindowDesc::new(|| ui_builder())
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu());

    AppLauncher::with_window(main_window)
        .delegate(Delegate::default())
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
