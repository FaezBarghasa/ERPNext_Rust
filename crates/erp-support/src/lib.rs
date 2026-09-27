pub mod gameplan;
pub mod sla;

pub use gameplan::{TaskStatus, WorkspaceManager, WorkspaceTask};
pub use sla::{
    SlaEscalationEvent, SlaWatchdog, SupportAgent, SupportError, SupportTicket, TicketPriority,
    TicketRouter, TicketStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_ticket_routing_workload_balancer() {
        let mut agents = vec![
            SupportAgent {
                agent_id: "agent_a".into(),
                name: "Agent A".into(),
                is_online: true,
                active_ticket_count: 3,
            },
            SupportAgent {
                agent_id: "agent_b".into(),
                name: "Agent B".into(),
                is_online: true,
                active_ticket_count: 1, // Lowest load
            },
            SupportAgent {
                agent_id: "agent_c".into(),
                name: "Agent C (Offline)".into(),
                is_online: false,
                active_ticket_count: 0,
            },
        ];

        let now = Utc::now();
        let mut ticket = SupportTicket {
            id: "TICK-001".into(),
            customer: "Acme Corp".into(),
            subject: "Printer issue".into(),
            priority: TicketPriority::Medium,
            status: TicketStatus::Open,
            assigned_to: None,
            created_at: now,
            response_deadline: now + Duration::hours(4),
        };

        // Assign ticket -> Should pick Agent B
        let assigned_agent =
            TicketRouter::assign_ticket(&mut ticket, &mut agents).expect("Assignment failed");

        assert_eq!(assigned_agent.agent_id, "agent_b");
        assert_eq!(assigned_agent.active_ticket_count, 2);
        assert_eq!(ticket.assigned_to, Some("agent_b".into()));
        assert_eq!(ticket.status, TicketStatus::Assigned);
    }

    #[test]
    fn test_sla_watchdog_deadline_escalation() {
        let now = Utc::now();
        let mut tickets = vec![
            // Expired ticket (deadline 1 hour ago)
            SupportTicket {
                id: "TICK-EXPIRED".into(),
                customer: "Beta Corp".into(),
                subject: "Server down".into(),
                priority: TicketPriority::High,
                status: TicketStatus::Open,
                assigned_to: None,
                created_at: now - Duration::hours(5),
                response_deadline: now - Duration::hours(1),
            },
            // Active ticket (deadline in 2 hours)
            SupportTicket {
                id: "TICK-ACTIVE".into(),
                customer: "Gamma Corp".into(),
                subject: "Password reset".into(),
                priority: TicketPriority::Low,
                status: TicketStatus::Open,
                assigned_to: None,
                created_at: now,
                response_deadline: now + Duration::hours(2),
            },
        ];

        let escalations = SlaWatchdog::evaluate_tickets(&mut tickets, now);

        assert_eq!(escalations.len(), 1);
        assert_eq!(escalations[0].ticket_id, "TICK-EXPIRED");
        assert_eq!(escalations[0].previous_priority, TicketPriority::High);
        assert_eq!(escalations[0].new_priority, TicketPriority::Urgent);

        assert_eq!(tickets[0].priority, TicketPriority::Urgent);
        assert_eq!(tickets[1].priority, TicketPriority::Low);
    }
}
