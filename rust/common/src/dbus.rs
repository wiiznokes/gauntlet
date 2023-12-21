use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{DeserializeDict, OwnedValue, SerializeDict, Type};

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusSearchResult {
    pub plugin_id: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusPlugin {
    pub plugin_id: String,
    pub plugin_name: String,
    pub enabled: bool,
    pub entrypoints: Vec<DBusEntrypoint>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusEntrypoint {
    pub entrypoint_id: String,
    pub entrypoint_name: String,
    pub enabled: bool,
}

#[derive(Debug, DeserializeDict, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct DBusUiWidget {
    pub widget_id: DbusUiWidgetId,
    pub widget_type: String,
    pub widget_properties: DBusUiPropertyContainer,
    pub widget_children: Vec<DBusUiWidget>,
}


#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewCreated {
    pub reconciler_mode: String,
    pub view_name: String,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewEvent {
    pub event_name: DbusUiEventName,
    pub widget_id: DbusUiWidgetId,
}

pub type DbusUiWidgetId = u32;
pub type DbusUiEventName = String;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusUiPropertyContainer(pub HashMap<String, (DBusUiPropertyValueType, OwnedValue)>);

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum DBusUiPropertyValueType {
    Function,
    String,
    Number,
    Bool,
}