

extern crate dupelib;



use druid::widget::{Align, Button, Flex, Label, Padding, WidgetExt};
use druid::{AppLauncher, Widget, WindowDesc};




use druid::commands;
use druid::platform_menus;
use druid::{
    Command, Data, FileDialogOptions, FileSpec, LocalizedString, MenuDesc, MenuItem, SysMods,
    AppDelegate, DelegateCtx, Env, FileInfo, Selector, Target, WindowId, Lens, Event
};








#[derive(Debug, Default)]
pub struct Delegate;

#[derive(Clone, Data, Default, Lens)]
pub struct AppState {
    // pub workspace: Workspace,
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: &Target,
        cmd: &Command,
        _data: &mut AppState,
        _env: &Env,
    ) -> bool {
        dbg!(cmd);
        match cmd.selector {
            druid::commands::OPEN_FILE => {
                let info = cmd.get_object::<FileInfo>().expect("api violation");
                dbg!(info);
                false
            }

            _ => true,
        }
    }

    fn event(
        &mut self,
        ctx: &mut DelegateCtx,
        window_id: WindowId,
        event: Event,
        data: &mut AppState,
        env: &Env
    ) -> Option<Event> {
        dbg!(event);
        None
    }

}


fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu());
    let data =  AppState {};
    dbg!("test");
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    Flex::column()
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
                    FileDialogOptions::new().select_directories().multi_selection(),
                ),
            )
            .hotkey(SysMods::Cmd, "o"),
        )
        .append_separator()
        .append(platform_menus::mac::file::close())
}