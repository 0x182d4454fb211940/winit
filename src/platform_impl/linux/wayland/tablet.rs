use std::collections::HashMap;

use once_cell::sync::OnceCell;
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    Dispatch, QueueHandle, WEnum,
};
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2,
    zwp_tablet_pad_v2::ZwpTabletPadV2,
    zwp_tablet_seat_v2::{
        ZwpTabletSeatV2, EVT_PAD_ADDED_OPCODE, EVT_TABLET_ADDED_OPCODE, EVT_TOOL_ADDED_OPCODE,
    },
    zwp_tablet_tool_v2::{Event, Type, ZwpTabletToolV2},
    zwp_tablet_v2::ZwpTabletV2,
};

use crate::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::{
        ElementState, Force, PenButton, PointerButton, PointerEvent, PointerId, Tilt, Tool,
        WindowEvent,
    },
    platform_impl::{wayland, wayland::DeviceId},
};

use super::state::WinitState;

pub struct TabletState {
    manager: ZwpTabletManagerV2,
    tools: HashMap<Tool, ToolState>,
}

impl TabletState {
    pub fn try_new(
        globals: &GlobalList,
        queue_handle: &QueueHandle<WinitState>,
    ) -> Option<TabletState> {
        let manager = globals.bind(&queue_handle, 1..=1, ()).ok()?;
        let tools = Default::default();
        Some(TabletState { manager, tools })
    }

    pub fn attach_seat(&mut self, seat: &WlSeat, queue_handle: &QueueHandle<WinitState>) {
        self.manager.get_tablet_seat(seat, queue_handle, ());
    }
}

impl Dispatch<ZwpTabletManagerV2, ()> for WinitState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletManagerV2,
        _event: <ZwpTabletManagerV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for WinitState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletSeatV2,
        _event: <ZwpTabletSeatV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }

    fn event_created_child(
        opcode: u16,
        queue_handle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_backend::client::ObjectData> {
        match opcode {
            EVT_TOOL_ADDED_OPCODE => queue_handle.make_data(ToolData::default()),
            EVT_TABLET_ADDED_OPCODE => queue_handle.make_data::<ZwpTabletV2, _>(DummyData {}),
            EVT_PAD_ADDED_OPCODE => queue_handle.make_data::<ZwpTabletPadV2, _>(DummyData {}),
            _ => panic!("unknown opcode {opcode}"),
        }
    }
}

#[derive(Default, Debug)]
struct ToolData {
    tool_type: OnceCell<Tool>,
}

impl ToolData {
    fn state_mut<'a>(&'a self, state: &'a mut WinitState) -> &'a mut ToolState {
        state
            .tablet
            .as_mut()
            .unwrap()
            .tools
            .get_mut(self.tool_type.get().unwrap())
            .unwrap()
    }
}

#[derive(Default, Debug)]
struct ToolState {
    surface: Option<WlSurface>,
    pressure: Option<u32>,
    tilt: Option<Tilt>,
    motion: Option<PhysicalPosition<f64>>,
    down: bool,
    up: bool,
}

impl Dispatch<ZwpTabletToolV2, ToolData> for WinitState {
    fn event(
        state: &mut Self,
        _proxy: &ZwpTabletToolV2,
        event: Event,
        data: &ToolData,
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            Event::ProximityIn { .. } | Event::ProximityOut => {
                println!("proximity event: {event:?}")
            }
            _ => (),
        }
    }
}

pub struct DummyData {}

impl Dispatch<ZwpTabletV2, DummyData> for WinitState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletV2,
        _event: <ZwpTabletV2 as wayland_client::Proxy>::Event,
        _data: &DummyData,
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletPadV2, DummyData> for WinitState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpTabletPadV2,
        _event: <ZwpTabletPadV2 as wayland_client::Proxy>::Event,
        _data: &DummyData,
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
