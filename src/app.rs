// SPDX-License-Identifier: MPL-2.0
use cosmic::iced::Subscription;
use cosmic::Task;
use cosmic::Element;
use cosmic::surface::Action as SurfaceAction;
use cosmic::iced::window;
use crate::systemstate::SystemState;
use cosmic::widget::{column, row, text, icon, divider};

const ID: &str = "com.github.aymen27k.sys_applet";

pub struct AppModel {
    core: cosmic::Core,
    system_state: SystemState,
    sys: sysinfo::System,
    components: sysinfo::Components,
    last_rx: u64,
    last_tx: u64,
    popup: Option<cosmic::iced::window::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Surface(SurfaceAction),
    PopupClosed(cosmic::iced::window::Id),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &cosmic::Core { &self.core }
    fn core_mut(&mut self) -> &mut cosmic::Core { &mut self.core }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let sys = sysinfo::System::new_all();
        let components = sysinfo::Components::new_with_refreshed_list();
        let networks = sysinfo::Networks::new_with_refreshed_list();
        
        // Seed the initial values so the first calculation isn't a massive spike
        let (initial_rx, initial_tx) = networks.get("enp3s0")
            .map(|d| (d.total_received(), d.total_transmitted()))
            .unwrap_or((0, 0));

        (
            AppModel { 
                core,
                system_state: SystemState::new(),
                sys,
                components,
                last_rx: initial_rx,
                last_tx: initial_tx,
                popup: None,
            },
            Task::none()
        )
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let have_popup = self.popup.clone();
        
        let btn = self.core.applet.icon_button("utilities-system-monitor-symbolic")
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(cosmic::surface::action::destroy_popup(id))
                } else {
                    Message::Surface(cosmic::surface::action::app_popup::<AppModel>(
                        move |state| {
                            let new_id = cosmic::iced::window::Id::unique();
                            state.popup = Some(new_id);
                            
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().expect("Main window exists"),
                                new_id,
                                None, 
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = cosmic::iced::Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            
                            popup_settings
                        },
                        Some(Box::new(move |state| {
                            let content = column![
                                row![
                                icon::icon(icon::from_name("utilities-system-monitor-symbolic").into())
                                    .size(24), // Making the header icon slightly larger
                                text::text("System Dashboard").size(20),
                            ]
                            .spacing(12)
                            .align_y(cosmic::iced::Alignment::Center),
                            
                            divider::horizontal::default(),
                                
                                // CPU Row
                                row![
                                    icon::icon(icon::from_name("computer-symbolic").into()), // Try 'cpu' instead of 'processor'
                                    text::text(format!(
                                        "CPU: {}% | {}°C", 
                                        state.system_state.cpu_load, 
                                        state.system_state.cpu_temp
                                    )),
                                ].spacing(10),
                                
                                

                                // RAM Row
                                row![
                                    icon::icon(icon::from_name("drive-harddisk-symbolic").into()), // Often used for memory/hardware
                                    text::text(format!(
                                        "RAM: {}GB / {}GB", 
                                        state.system_state.ram_usage.0, 
                                        state.system_state.ram_usage.1
                                    )),
                                ].spacing(10),
                                
                                // GPU Row
                                row![
                                    icon::icon(icon::from_name("input-gaming-symbolic").into()),
                                    text::text(format!("GPU: {}", state.system_state.gpu_temp)),
                                ].spacing(10),
                                
                                // Network Row
                                row![
                                    icon::icon(icon::from_name("network-transmit-receive-symbolic").into()),
                                    text::text(format!(
                                        "↓{} | ↑{}", 
                                        state.system_state.format_net_down(), 
                                        state.system_state.format_net_up()
                                    )),
                                ]
                                .spacing(10)
                                .align_y(cosmic::iced::Alignment::Center),
                                
                                // Audio Row
                                row![
                                    icon::icon(icon::from_name("audio-card-symbolic").into()),
                                    text::text(format!("Audio: {}", state.system_state.audio_rate)),
                                ].spacing(10),
                            ]
                            .spacing(15)
                            .padding(20);

                            Element::from(state.core.applet.popup_container(content))
                                .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        Element::from(self.core.applet.applet_tooltip(
            btn,
            "System Monitor",
            self.popup.is_some(),
            |a| Message::Surface(a),
            None,
        ))
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        cosmic::widget::text("").into()
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Tick => {
                // Pass the persistent last_rx/tx from 'self' into the collector
                self.system_state = crate::systemstate::collect_system_data(
                    &mut self.sys, 
                    &self.components, 
                    &mut self.last_rx,
                    &mut self.last_tx
                );
                Task::none()
            }
            
            Message::Surface(a) => {
                cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ))
            }

            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::time::every(std::time::Duration::from_millis(1000))
            .map(|_| Message::Tick)
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
    
}