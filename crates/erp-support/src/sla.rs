use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Support and ticketing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SupportError {
    /// No online support agents available.
    #[error("No online support agents available for assignment")]
    NoOnlineAgentsAvailable,
    /// Ticket not found.
    #[error("Ticket not found: {0}")]
    TicketNotFound(String),
}

/// Support ticket priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Operational status of support tickets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Open,
    Assigned,
    Resolved,
    Closed,
}

/// Support Ticket record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportTicket {
    pub id: String,
    pub customer: String,
    pub subject: String,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub response_deadline: DateTime<Utc>,
}

/// Support technician agent representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportAgent {
    pub agent_id: String,
    pub name: String,
    pub is_online: bool,
    pub active_ticket_count: usize,
}

/// Omnichannel Ticket Routing & Workload Balancer (Milestone 4.1).
pub struct TicketRouter;

impl TicketRouter {
    /// Assigns an incoming ticket to the online technician with the lowest active workload.
    pub fn assign_ticket<'a>(
        ticket: &mut SupportTicket,
        agents: &'a mut [SupportAgent],
    ) -> Result<&'a mut SupportAgent, SupportError> {
        let least_loaded = agents
            .iter_mut()
            .filter(|a| a.is_online)
            .min_by_key(|a| a.active_ticket_count);

        match least_loaded {
            Some(agent) => {
                agent.active_ticket_count += 1;
                ticket.assigned_to = Some(agent.agent_id.clone());
                ticket.status = TicketStatus::Assigned;
                Ok(agent)
            }
            None => Err(SupportError::NoOnlineAgentsAvailable),
        }
    }
}

/// Escalation event generated upon SLA breach.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlaEscalationEvent {
    pub ticket_id: String,
    pub previous_priority: TicketPriority,
    pub new_priority: TicketPriority,
    pub breached_at: DateTime<Utc>,
}

/// Real-Time SLA Monitoring Watchdog (Milestone 4.2).
pub struct SlaWatchdog;

impl SlaWatchdog {
    /// Evaluates unresolved tickets against their SLA response deadlines and escalates breached tickets.
    #[must_use]
    pub fn evaluate_tickets(
        tickets: &mut [SupportTicket],
        now: DateTime<Utc>,
    ) -> Vec<SlaEscalationEvent> {
        let mut escalations = Vec::new();

        for ticket in tickets.iter_mut() {
            if matches!(ticket.status, TicketStatus::Open | TicketStatus::Assigned)
                && now > ticket.response_deadline
                && ticket.priority != TicketPriority::Urgent
            {
                let prev = ticket.priority;
                ticket.priority = TicketPriority::Urgent;

                escalations.push(SlaEscalationEvent {
                    ticket_id: ticket.id.clone(),
                    previous_priority: prev,
                    new_priority: TicketPriority::Urgent,
                    breached_at: now,
                });
            }
        }

        escalations
    }
}
