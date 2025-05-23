use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    dark_mode: bool,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();
    
    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            dark_mode: false,
        }
   }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                true // Re-render the component
            }
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://api.dicebear.com/9.x/notionists-neutral/svg"
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_dark_mode = ctx.link().callback(|_| Msg::ToggleDarkMode);
        
        // Define theme classes based on dark mode state
        let (bg_primary, bg_secondary, bg_tertiary, text_primary, text_secondary, border_color) = if self.dark_mode {
            ("bg-gray-900", "bg-gray-800", "bg-gray-700", "text-white", "text-gray-300", "border-gray-600")
        } else {
            ("bg-white", "bg-gray-100", "bg-white", "text-gray-900", "text-gray-600", "border-gray-300")
        };

        html! {
            <div class={format!("flex w-screen {}", if self.dark_mode { "bg-gray-900" } else { "bg-white" })}>
                // Sidebar
                <div class={format!("flex-none w-56 h-screen {}", bg_secondary)}>
                    // Header with dark mode toggle
                    <div class={format!("flex justify-between items-center text-xl p-3 {}", text_primary)}>
                        <span>{"Users"}</span>
                        <button onclick={toggle_dark_mode} class={format!("p-2 rounded-lg hover:{} transition-colors", if self.dark_mode { "bg-gray-600" } else { "bg-gray-200" })}>
                            if self.dark_mode {
                                // Sun icon for light mode
                                <svg class="w-5 h-5 fill-yellow-400" viewBox="0 0 24 24">
                                    <path d="M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1zM5.99 4.58c-.39-.39-1.03-.39-1.41 0-.39.39-.39 1.03 0 1.41L6.7 7.1c.39.39 1.03.39 1.41 0 .39-.39.39-1.03 0-1.41L5.99 4.58zM18.36 16.95c-.39-.39-1.03-.39-1.41 0-.39.39-.39 1.03 0 1.41l2.12 2.12c.39.39 1.03.39 1.41 0 .39-.39.39-1.03 0-1.41l-2.12-2.12zm0-11.24l2.12-2.12c.39-.39.39-1.03 0-1.41-.39-.39-1.03-.39-1.41 0l-2.12 2.12c-.39.39-.39 1.03 0 1.41.39.39 1.03.39 1.41 0zm-11.24 11.24L5 18.36c-.39.39-.39 1.03 0 1.41.39.39 1.03.39 1.41 0L8.54 17.65c.39-.39.39-1.03 0-1.41-.39-.39-1.03-.39-1.41 0z"/>
                                </svg>
                            } else {
                                // Moon icon for dark mode
                                <svg class="w-5 h-5 fill-gray-600" viewBox="0 0 24 24">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                                </svg>
                            }
                        </button>
                    </div>
                    
                    // Users list
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class={format!("flex m-3 {} rounded-lg p-2 transition-colors", bg_tertiary)}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class={format!("flex text-xs justify-between {}", text_primary)}>
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                
                // Main chat area
                <div class={format!("grow h-screen flex flex-col {}", bg_primary)}>
                    // Chat header
                    <div class={format!("w-full h-14 border-b-2 {}", border_color)}>
                        <div class={format!("text-xl p-3 {}", text_primary)}>{"💬 Chat!"}</div>
                    </div>
                    
                    // Messages area
                    <div class={format!("w-full grow overflow-auto border-b-2 {}", border_color)}>
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class={format!("flex items-end w-3/6 {} m-8 rounded-tl-lg rounded-tr-lg rounded-br-lg transition-colors", bg_secondary)}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class={format!("text-sm {}", text_primary)}>
                                                {m.from.clone()}
                                            </div>
                                            <div class={format!("text-xs {}", text_secondary)}>
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Input area
                    <div class="w-full h-14 flex px-3 items-center">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Message" 
                            class={format!("block w-full py-2 pl-4 mx-3 {} rounded-full outline-none focus:{} transition-colors", 
                                bg_secondary, 
                                text_primary
                            )} 
                            name="message" 
                            required=true 
                        />
                        <button 
                            onclick={submit} 
                            class="p-3 shadow-sm bg-blue-600 hover:bg-blue-700 w-10 h-10 rounded-full flex justify-center items-center transition-colors"
                        >
                            <svg fill="white" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-4 h-4">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}