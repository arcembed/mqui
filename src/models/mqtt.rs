use std::time::SystemTime;

pub(crate) const MAX_STORED_MESSAGES: usize = 1000;

#[derive(Clone, Debug)]
pub(crate) struct MqttLoginData {
    pub(crate) name: String,
    pub(crate) broker: String,
    pub(crate) port: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) client_id: String,
    pub(crate) keep_alive_secs: u16,
    pub(crate) testament_and_last_will: String,
    pub(crate) testament_topic: String,
    pub(crate) testament_qos: u8,
    pub(crate) testament_retain: bool,
}

impl Default for MqttLoginData {
    fn default() -> Self {
        Self {
            name: String::new(),
            broker: String::new(),
            port: String::new(),
            username: String::new(),
            password: String::new(),
            client_id: String::new(),
            keep_alive_secs: 60,
            testament_and_last_will: String::new(),
            testament_topic: String::new(),
            testament_qos: 0,
            testament_retain: false,
        }
    }
}

impl MqttLoginData {
    pub(crate) fn broker_addr(&self) -> String {
        let broker = self.broker.trim();
        if broker.is_empty() {
            return "127.0.0.1:1883".to_string();
        }

        if broker.contains(':') {
            broker.to_string()
        } else {
            let port = self.port.trim();
            let port = if port.is_empty() { "1883" } else { port };
            format!("{broker}:{port}")
        }
    }

    pub(crate) fn username_opt(&self) -> Option<&str> {
        let value = self.username.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn password_opt(&self) -> Option<&str> {
        let value = self.password.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn testament_and_last_will_opt(&self) -> Option<&str> {
        let value = self.testament_and_last_will.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn testament_topic_opt(&self) -> Option<&str> {
        let value = self.testament_topic.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(crate) fn effective_client_id(&self, tab_id: u64) -> String {
        let value = self.client_id.trim();
        if value.is_empty() {
            format!("mqui-client-{}-{tab_id}", std::process::id())
        } else {
            value.to_string()
        }
    }

    pub(crate) fn effective_keep_alive_secs(&self) -> u16 {
        self.keep_alive_secs.max(1)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SubscriptionEntry {
    pub(crate) topic: String,
    pub(crate) qos: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct ReceivedMessage {
    pub(crate) timestamp: SystemTime,
    pub(crate) topic: String,
    pub(crate) qos: u8,
    pub(crate) retain: bool,
    pub(crate) payload: Vec<u8>,
}
