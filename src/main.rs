extern crate dupelib;

use druid::commands;
use druid::platform_menus;

use druid::AppDelegate;
use druid::DelegateCtx;
use druid::Target;

use druid::{AppLauncher, Widget, WindowDesc};
use druid::{
    Command, Data, Env, FileDialogOptions, FileInfo, Lens, LocalizedString, MenuDesc, MenuItem, SysMods,
};
#[derive(Debug, Default)]
pub struct Delegate;

use druid::widget::{Button, Flex, Label};
use druid::WidgetExt;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Data, Default, Lens)]
pub struct AppState {
    pub paths: Arc<Mutex<Vec<PathBuf>>>,
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
                    data.paths.try_lock().unwrap().push(pathbuf);
                    dbg!(&data.paths);
                    false
                }
                _ => true
            }
    }
}


fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let text = LocalizedString::new("hello-counter");
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(|_ctx, _data, _env| {
            dbg!("clicked");
        })
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
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
                    FileDialogOptions::new()
                        .select_directories()
                        .multi_selection(),
                ),
            )
            .hotkey(SysMods::Cmd, "o"),
        )
        .append_separator()
        .append(platform_menus::mac::file::close())
}
