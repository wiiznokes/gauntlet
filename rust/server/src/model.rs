use std::collections::HashMap;

use deno_core::serde_v8;
use serde::{Deserialize, Serialize};

use common::model::PropertyValue;
use common::rpc::{RpcUiPropertyValue, RpcUiWidget, RpcUiWidgetId};
use common::rpc::rpc_ui_property_value::Value;

#[derive(Debug)]
pub enum JsUiResponseData {
    Nothing
}

#[derive(Debug)]
pub enum JsUiRequestData {
    ReplaceView {
        render_location: JsRenderLocation,
        top_level_view: bool,
        container: IntermediateUiWidget,
    },
    ClearInlineView
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum JsRenderLocation {
    InlineView,
    View
}


pub type UiWidgetId = u32;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsUiEvent {
    OpenView {
        #[serde(rename = "frontend")]
        frontend: String,
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    RunCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    ViewEvent {
        #[serde(rename = "widgetId")]
        widget_id: UiWidgetId,
        #[serde(rename = "eventName")]
        event_name: String,
        #[serde(rename = "eventArguments")]
        event_arguments: Vec<JsPropertyValue>,
    },
    PluginCommand {
        #[serde(rename = "commandType")]
        command_type: String,
    },
    OpenInlineView {
        #[serde(rename = "text")]
        text: String,
    }
}

// FIXME this could have been serde_v8::AnyValue but it doesn't support undefined, make a pr?
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsPropertyValue {
    String {
        value: String
    },
    Number {
        value: f64
    },
    Bool {
        value: bool
    },
    Undefined,
}

#[derive(Deserialize, Serialize)]
pub struct JsUiWidget<'a> {
    #[serde(rename = "widgetId")]
    pub widget_id: UiWidgetId,
    #[serde(rename = "widgetType")]
    pub widget_type: String,
    #[serde(rename = "widgetProperties")]
    pub widget_properties: HashMap<String, serde_v8::Value<'a>>,
    #[serde(rename = "widgetChildren")]
    pub widget_children: Vec<JsUiWidget<'a>>,
}

#[derive(Debug)]
pub enum IntermediateUiEvent {
    OpenView {
        frontend: String,
        entrypoint_id: String
    },
    RunCommand {
        entrypoint_id: String
    },
    ViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<PropertyValue>,
    },
    PluginCommand {
        command_type: String,
    },
    OpenInlineView {
        text: String,
    }
}

#[derive(Debug)]
pub struct IntermediateUiWidget {
    pub widget_id: UiWidgetId,
    pub widget_type: String,
    pub widget_properties: HashMap<String, PropertyValue>,
    pub widget_children: Vec<IntermediateUiWidget>,
}

impl From<IntermediateUiWidget> for RpcUiWidget {
    fn from(value: IntermediateUiWidget) -> Self {
        let children = value.widget_children.into_iter()
            .map(|child| child.into())
            .collect::<Vec<RpcUiWidget>>();

        let widget_id = RpcUiWidgetId {
            value: value.widget_id
        };

        Self {
            widget_id: Some(widget_id),
            widget_type: value.widget_type,
            widget_properties: from_intermediate_to_rpc_properties(value.widget_properties),
            widget_children: children
        }
    }
}

pub fn from_rpc_to_intermediate_value(value: RpcUiPropertyValue) -> Option<PropertyValue> {
    let value = match value.value? {
        Value::Undefined(_) => PropertyValue::Undefined,
        Value::String(value) => PropertyValue::String(value),
        Value::Number(value) => PropertyValue::Number(value),
        Value::Bool(value) => PropertyValue::Bool(value)
    };

    Some(value)
}


fn from_intermediate_to_rpc_properties(value: HashMap<String, PropertyValue>) -> HashMap<String, RpcUiPropertyValue> {
    value.into_iter()
        .filter_map(|(key, value)| {
            match value {
                PropertyValue::String(value) => Some((key, RpcUiPropertyValue { value: Some(Value::String(value)) })),
                PropertyValue::Number(value) => Some((key, RpcUiPropertyValue { value: Some(Value::Number(value)) })),
                PropertyValue::Bool(value) => Some((key, RpcUiPropertyValue { value: Some(Value::Bool(value)) })),
                PropertyValue::Undefined => None
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub enum PluginEntrypointType {
    Command,
    View,
    InlineView,
}

pub fn entrypoint_to_str(value: PluginEntrypointType) -> &'static str {
    match value {
        PluginEntrypointType::Command => "command",
        PluginEntrypointType::View => "view",
        PluginEntrypointType::InlineView => "inline-view",
    }
}

pub fn entrypoint_from_str(value: &str) -> PluginEntrypointType {
    match value {
        "command" => PluginEntrypointType::Command,
        "view" => PluginEntrypointType::View,
        "inline-view" => PluginEntrypointType::InlineView,
        _ => {
            panic!("index contains illegal entrypoint_type: {}", value)
        }
    }
}