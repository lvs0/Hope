//! Library entry point for hope-hal
//!
//! Exposes the HAL functionality as a library for other Hope OS components.

pub mod db;
pub mod detection;
pub mod drivers;
pub mod notifications;

pub use db::{DriverDatabase, DriverInfo};
pub use detection::{parse_lsusb_output, request_cloud_lookup, UdevEvent, UdevMonitor};
pub use drivers::{install_driver, is_module_loaded};
pub use notifications::{spawn_notification_system, Action, Notification, NotificationSender};

#[cfg(test)]
mod tests {
    use crate::notifications::Notification;

    #[test]
    fn test_simple_notification() {
        let n = Notification::simple("Titre", "Corps du message");
        assert_eq!(n.actions.len(), 1);
        assert_eq!(n.actions[0].id, "ok");
    }

    #[test]
    fn test_question_notification() {
        let n = Notification::question(
            "Veux-tu redémarrer ?",
            "Un pilote a été installé.",
            vec![("Oui", "yes"), ("Plus tard", "later")],
        );
        assert_eq!(n.actions.len(), 2);
    }

    #[test]
    fn test_notification_validate_rejects_jargon() {
        let n = Notification {
            title: "Kernel".into(),
            body: "The kernel module loaded".into(),
            actions: vec![],
        };
        assert!(n.validate().is_err());
    }

    #[test]
    fn test_notification_validate_rejects_too_many_buttons() {
        let n = Notification {
            title: "Test".into(),
            body: "Choose one".into(),
            actions: vec![
                crate::notifications::Action { label: "A".into(), id: "a".into() },
                crate::notifications::Action { label: "B".into(), id: "b".into() },
                crate::notifications::Action { label: "C".into(), id: "c".into() },
                crate::notifications::Action { label: "D".into(), id: "d".into() },
            ],
        };
        assert!(n.validate().is_err());
    }
}
