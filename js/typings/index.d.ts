declare namespace Deno {
    const internal: unique symbol;
    function inspect(value: unknown, options?: InspectOptions): string
}

interface InspectOptions {
    depth: number
}

interface Deno {
    [Deno.internal]: {
        core: {
            opAsync: (op: string) => Promise<PluginEvent>
            ops: InternalApi
        }
    };
}

type PluginEvent = ViewEvent | ViewCreated | ViewDestroyed | PluginCommand

type ViewEvent = {
    type: "ViewEvent"
    eventName: string
    widget: Instance
}

type ViewCreated = {
    type: "ViewCreated"
    reconcilerMode: string
    viewName: string
}

type ViewDestroyed = {
    type: "ViewDestroyed"
}

type PluginCommand = {
    type: "PluginCommand"
    commandType: "stop"
}

type UiWidget = UiWidgetBase & {
    hostContext: RootContext
}
type RootUiWidget = UiWidgetBase & RootContext

type UiWidgetBase = {
    widgetId: number,
    widgetType: string,
    widgetProperties: Props,
    widgetChildren: UiWidget[],
}

type ComponentType = string;
type Props = { [key: string]: any };
type PropsWithChildren = { children?: UiWidget[] } & Props;

type RootContext = { nextId: number }
type Instance = UiWidget
type TextInstance = UiWidget
type ChildSet = (Instance | TextInstance)[]
type UpdatePayload = string[];

type SuspenseInstance = never;

interface InternalApi {
    op_log_trace(target: string, message: string): void;
    op_log_debug(target: string, message: string): void;
    op_log_info(target: string, message: string): void;
    op_log_warn(target: string, message: string): void;
    op_log_error(target: string, message: string): void;

    op_react_call_event_listener(instance: Instance, eventName: string): void;

    op_react_clone_instance(instance: Instance, updatePayload: UpdatePayload, type: ComponentType, oldProps: PropsWithChildren, newProps: PropsWithChildren, keepChildren: boolean): Instance;

    op_react_replace_container_children(container: Instance, newChildren: ChildSet): void;
}
