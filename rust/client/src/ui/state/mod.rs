mod main_view;
mod plugin_view;

use crate::ui::client_context::ClientContext;
use crate::ui::scroll_handle::{ScrollHandle, ESTIMATED_MAIN_LIST_ITEM_HEIGHT};
pub use crate::ui::state::main_view::MainViewState;
pub use crate::ui::state::plugin_view::PluginViewState;
use crate::ui::AppMsg;
use common::model::{EntrypointId, PhysicalShortcut, PluginId, SearchResult};
use iced::widget::text_input;
use iced::widget::text_input::focus;
use iced::Command;
use std::collections::HashMap;
use std::sync::{Arc, RwLock as StdRwLock};

pub enum GlobalState {
    MainView {
        // logic
        search_field_id: text_input::Id,

        // ephemeral state
        focused_search_result: ScrollHandle<SearchResult>,

        // state
        client_context: Arc<StdRwLock<ClientContext>>,
        sub_state: MainViewState,
        pending_plugin_view_data: Option<PluginViewData>,
        pending_plugin_view_loading_bar: LoadingBarState,
    },
    ErrorView {
        error_view: ErrorViewData,
    },
    PluginView {
        // logic
        client_context: Arc<StdRwLock<ClientContext>>,

        // state
        plugin_view_data: PluginViewData,
        sub_state: PluginViewState,
    },
}

#[derive(Clone)]
pub struct PluginViewData {
    pub top_level_view: bool,
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub action_shortcuts: HashMap<String, PhysicalShortcut>,
}

pub enum ErrorViewData {
    PreferenceRequired {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    PluginError {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    BackendTimeout,
    UnknownError {
        display: String
    },
}

#[derive(Debug, Clone)]
pub enum LoadingBarState {
    Off,
    Pending,
    On
}


impl GlobalState {
    pub fn new(search_field_id: text_input::Id, client_context: Arc<StdRwLock<ClientContext>>) -> GlobalState {
        GlobalState::MainView {
            search_field_id,
            focused_search_result: ScrollHandle::new(true, ESTIMATED_MAIN_LIST_ITEM_HEIGHT, 7),
            sub_state: MainViewState::new(),
            pending_plugin_view_data: None,
            client_context,
            pending_plugin_view_loading_bar: LoadingBarState::Off,
        }
    }

    pub fn new_error(error_view_data: ErrorViewData) -> GlobalState {
        GlobalState::ErrorView {
            error_view: error_view_data,
        }
    }

    pub fn new_plugin(plugin_view_data: PluginViewData, client_context: Arc<StdRwLock<ClientContext>>) -> GlobalState {
        GlobalState::PluginView {
            client_context,
            plugin_view_data,
            sub_state: PluginViewState::new(),
        }
    }

    pub fn initial(prev_global_state: &mut GlobalState, client_context: Arc<StdRwLock<ClientContext>>) -> Command<AppMsg> {
        let search_field_id = text_input::Id::unique();

        *prev_global_state = GlobalState::new(search_field_id.clone(), client_context);

        Command::batch([
            focus(search_field_id),
            Command::perform(async {}, |_| AppMsg::UpdateSearchResults),
        ])
    }

    pub fn error(prev_global_state: &mut GlobalState, error_view_data: ErrorViewData) -> Command<AppMsg> {
        *prev_global_state = GlobalState::new_error(error_view_data);

        Command::none()
    }

    pub fn plugin(prev_global_state: &mut GlobalState, plugin_view_data: PluginViewData, client_context: Arc<StdRwLock<ClientContext>>) -> Command<AppMsg> {
        *prev_global_state = GlobalState::new_plugin(plugin_view_data, client_context);

        Command::none()
    }
}

pub trait Focus<T> {
    fn primary(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn secondary(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn back(&mut self) -> Command<AppMsg>;
    fn next(&mut self) -> Command<AppMsg>;
    fn previous(&mut self) -> Command<AppMsg>;
    fn up(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn down(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn left(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn right(&mut self, focus_list: &[T]) -> Command<AppMsg>;
}

impl Focus<SearchResult> for GlobalState {
    fn primary(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if let Some(search_result) = focused_search_result.get(focus_list) {
                            let search_result = search_result.clone();
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithFocus { search_result })
                        } else {
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus)
                        }
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item, .. } => {
                        match focused_action_item.index {
                            None => Command::none(),
                            Some(widget_id) => {
                                if let Some(search_result) = focused_search_result.get(&focus_list) {
                                    let search_result = search_result.clone();
                                    Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id })
                                } else {
                                    Command::none()
                                }
                            }
                        }
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        match focused_action_item.index {
                            None => Command::none(),
                            Some(widget_id) => {
                                Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewInlineViewPanelKeyboardWithFocus { widget_id })
                            }
                        }
                    }
                }
            }
            GlobalState::PluginView { sub_state, client_context, .. } => {
                let client_context = client_context.read().expect("lock is poisoned");

                let action_ids = client_context.get_action_ids();

                match sub_state {
                    PluginViewState::None => {
                        if let Some(widget_id) = action_ids.get(0) {
                            let widget_id = *widget_id;
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionPluginViewNoPanelKeyboardWithFocus { widget_id })
                        } else {
                            Command::none()
                        }
                    },
                    PluginViewState::ActionPanel { focused_action_item, .. } => {
                        if let Some(widget_id) = focused_action_item.get(&action_ids) {
                            let widget_id = *widget_id;
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionPluginViewAnyPanelKeyboardWithFocus { widget_id })
                        } else {
                            Command::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none()
        }
    }

    fn secondary(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if let Some(search_result) = focused_search_result.get(focus_list) {
                            let search_result = search_result.clone();
                            Command::perform(async {}, move |_| AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result })
                        } else {
                            Command::perform(async {}, move |_| AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus)
                        }
                    }
                    MainViewState::SearchResultActionPanel { .. } | MainViewState::InlineViewActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Command::none()
                    }
                }
            }
            GlobalState::PluginView { sub_state, client_context, .. } => {
                let client_context = client_context.read().expect("lock is poisoned");

                let action_ids = client_context.get_action_ids();

                match sub_state {
                    PluginViewState::None => {
                        if let Some(widget_id) = action_ids.get(1) {
                            let widget_id = *widget_id;
                            Command::perform(async {}, move |_| AppMsg::OnSecondaryActionPluginViewNoPanelKeyboardWithFocus { widget_id })
                        } else {
                            Command::none()
                        }
                    },
                    PluginViewState::ActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Command::none()
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none()
        }
    }

    fn back(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        Command::perform(async {}, |_| AppMsg::HideWindow)
                    }
                    MainViewState::SearchResultActionPanel { .. } => {
                        MainViewState::initial(sub_state);
                        Command::none()
                    }
                    MainViewState::InlineViewActionPanel { .. } => {
                        MainViewState::initial(sub_state);
                        Command::none()
                    }
                }
            }
            GlobalState::PluginView {
                plugin_view_data: PluginViewData {
                    top_level_view,
                    plugin_id,
                    entrypoint_id,
                    ..
                },
                sub_state,
                client_context,
                ..
            } => {
                match sub_state {
                    PluginViewState::None => {
                        if *top_level_view {
                            let plugin_id = plugin_id.clone();

                            let client_context = client_context.clone();

                            Command::batch([
                                Command::perform(async {}, |_| AppMsg::ClosePluginView(plugin_id)),
                                GlobalState::initial(self, client_context)
                            ])
                        } else {
                            let plugin_id = plugin_id.clone();
                            let entrypoint_id = entrypoint_id.clone();
                            Command::perform(async {}, |_| AppMsg::OpenPluginView(plugin_id, entrypoint_id))
                        }
                    }
                    PluginViewState::ActionPanel { .. } => {
                        Command::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                    }
                }
            }
            GlobalState::ErrorView { .. } => {
                Command::perform(async {}, |_| AppMsg::HideWindow)
            }
        }
    }
    fn next(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn previous(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn up(&mut self, _focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        focused_search_result.focus_previous()
                            .unwrap_or_else(|| Command::none())
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Command::none())
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Command::none())
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none(),
            GlobalState::PluginView { sub_state, client_context, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        client_context.focus_up()
                    },
                    PluginViewState::ActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Command::none())
                    }
                }
            },
        }
    }
    fn down(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, client_context, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if focus_list.len() != 0 {
                            focused_search_result.focus_next(focus_list.len())
                                .unwrap_or_else(|| Command::none())
                        } else {
                            Command::none()
                        }
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item } => {
                        if let Some(search_item) = focused_search_result.get(focus_list) {
                            if search_item.entrypoint_actions.len() != 0 {
                                focused_action_item.focus_next(search_item.entrypoint_actions.len() + 1)
                                    .unwrap_or_else(|| Command::none())
                            } else {
                                Command::none()
                            }
                        } else {
                            Command::none()
                        }
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        match client_context.get_first_inline_view_action_panel() {
                            Some(action_panel) => {
                                if action_panel.action_count() != 0 {
                                    focused_action_item.focus_next(action_panel.action_count())
                                        .unwrap_or_else(|| Command::none())
                                } else {
                                    Command::none()
                                }
                            }
                            None => Command::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none(),
            GlobalState::PluginView { sub_state, client_context, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        client_context.focus_down()
                    },
                    PluginViewState::ActionPanel { focused_action_item } => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        let action_ids = client_context.get_action_ids();

                        if action_ids.len() != 0 {
                            focused_action_item.focus_next(action_ids.len())
                                .unwrap_or_else(|| Command::none())
                        } else {
                            Command::none()
                        }
                    }
                }
            }
        }
    }
    fn left(&mut self, _focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::PluginView { client_context, sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        client_context.focus_left()
                    }
                    PluginViewState::ActionPanel { .. } => Command::none()
                }
            },
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn right(&mut self, _focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::PluginView { client_context, sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        client_context.focus_right()
                    }
                    PluginViewState::ActionPanel { .. } => Command::none()
                }
            },
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
}
