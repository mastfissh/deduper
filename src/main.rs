extern crate dupelib;

use druid::commands;
use druid::platform_menus;
use druid::widget::prelude::*;

use druid::{AppLauncher, Widget, WindowDesc};
use druid::{
    BoxConstraints, Command, Data, Env, Event, FileDialogOptions, FileInfo, LayoutCtx, Lens,
    LifeCycle, LocalizedString, MenuDesc, MenuItem, PaintCtx, Size, SysMods,
};

use druid::widget::{Button, Flex, Label};
use druid::WidgetExt;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Data, Default, Lens)]
pub struct AppState {
    pub paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl Widget<AppState> for AppState {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        match event {
            Event::Command(cmd) => match cmd.selector {
                druid::commands::OPEN_FILE => {
                    let info = cmd.get_object::<FileInfo>().expect("api violation");
                    let pathbuf = info.path().clone().to_path_buf();
                    data.paths.try_lock().unwrap().push(pathbuf);
                    dbg!(&data.paths);
                }
                _ => {}
            },
            _ => {}
        }
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

fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let text = LocalizedString::new("hello-counter");
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(|_ctx, data, _env| {
            dbg!("clicked");
        })
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}

fn main() {
    let main_window = WindowDesc::new(|| AppState::default())
        .title(LocalizedString::new("Dupe Detector"))
        .menu(make_menu());

    AppLauncher::with_window(main_window)
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
