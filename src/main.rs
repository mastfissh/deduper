extern crate dupelib;

use druid::commands;
use druid::platform_menus;
use druid::widget::prelude::*;

use druid::{
    AppDelegate, BoxConstraints, Command, Data, DelegateCtx, Env, Event, FileDialogOptions,
    FileInfo, FileSpec, LayoutCtx, Lens, LifeCycle, LocalizedString, MenuDesc, MenuItem, PaintCtx,
    Selector, Size, SysMods, Target, WindowId,
};
use druid::{AppLauncher, Widget, WindowDesc};

#[derive(Debug, Default)]
pub struct Delegate;

#[derive(Clone, Data, Default, Lens, Copy)]
pub struct AppState {
    // pub workspace: Workspace,
}

impl Widget<AppState> for AppState {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, _data: &mut AppState, _env: &Env) {
        dbg!(event);
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
        dbg!(event);
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {
        dbg!("t");
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        bc.constrain((900.0, 900.0))
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppState, _env: &Env) {}
}

fn main() {
    let main_window = WindowDesc::new(|| AppState {})
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu());

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(AppState {})
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
