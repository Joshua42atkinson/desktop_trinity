use super::*;

pub fn get_demo_workflow() -> Workflow {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Node IDs
    let trigger_id = Uuid::new_v4();
    let researcher_id = Uuid::new_v4();
    let writer_id = Uuid::new_v4();
    let title_id = Uuid::new_v4();

    // 1. Trigger
    nodes.insert(
        trigger_id,
        WorkflowNode {
            id: trigger_id,
            label: "Start: 'AI Agents Future'".to_string(),
            kind: NodeKind::Trigger(TriggerType::Manual),
            position: Vec2::new(100.0, 300.0),
        },
    );

    // 2. Researcher Agent
    nodes.insert(
        researcher_id,
        WorkflowNode {
            id: researcher_id,
            label: "Researcher".to_string(),
            kind: NodeKind::Agent(AgentConfig {
                role_name: "Research".to_string(),
                system_prompt_override: Some(
                    "You are a tech trend analyst. List 3 key trends in AI Agents.".to_string(),
                ),
                model_override: None,
            }),
            position: Vec2::new(400.0, 300.0),
        },
    );

    // 3. Script Writer Agent
    nodes.insert(
        writer_id,
        WorkflowNode {
            id: writer_id,
            label: "Script Writer".to_string(),
            kind: NodeKind::Agent(AgentConfig {
                role_name: "Writer".to_string(),
                system_prompt_override: Some(
                    "Convert these trends into a 30s YouTube Short script. Engaging tone."
                        .to_string(),
                ),
                model_override: None,
            }),
            position: Vec2::new(700.0, 300.0),
        },
    );

    // 4. Title Generator Agent
    nodes.insert(
        title_id,
        WorkflowNode {
            id: title_id,
            label: "Title Expert".to_string(),
            kind: NodeKind::Agent(AgentConfig {
                role_name: "Writer".to_string(),
                system_prompt_override: Some(
                    "Create 3 viral clickbait titles for this script.".to_string(),
                ),
                model_override: None,
            }),
            position: Vec2::new(1000.0, 300.0),
        },
    );

    // Edges
    edges.push(WorkflowEdge {
        id: Uuid::new_v4(),
        source: trigger_id,
        target: researcher_id,
        label: None,
    });
    edges.push(WorkflowEdge {
        id: Uuid::new_v4(),
        source: researcher_id,
        target: writer_id,
        label: None,
    });
    edges.push(WorkflowEdge {
        id: Uuid::new_v4(),
        source: writer_id,
        target: title_id,
        label: None,
    });

    Workflow {
        id: Uuid::new_v4(),
        name: "YouTube Script Automation".to_string(),
        nodes,
        edges,
        created_at: 0.0,
    }
}
