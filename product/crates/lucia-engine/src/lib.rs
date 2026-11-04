use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(String);

impl ActionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRole {
    Container,
    Text,
    Button,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accessibility {
    pub role: SemanticRole,
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticTree {
    pub root: SemanticNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNode {
    pub id: NodeId,
    pub accessibility: Accessibility,
    pub size: Option<LogicalSize>,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Container {
        layout: Layout,
        children: Vec<SemanticNode>,
    },
    Text {
        text: String,
    },
    Button {
        label: String,
        action: ActionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyNodeId,
    DuplicateNodeId(NodeId),
    EmptyAccessibleName(NodeId),
    EmptyAccessibleLabel(NodeId),
    RoleMismatch {
        node_id: NodeId,
        expected: SemanticRole,
        actual: SemanticRole,
    },
    EmptyButtonLabel(NodeId),
    EmptyActionId(NodeId),
    InvalidLogicalSize(NodeId),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNodeId => write!(formatter, "node ID must not be empty"),
            Self::DuplicateNodeId(node_id) => {
                write!(formatter, "duplicate node ID: {}", node_id.as_str())
            }
            Self::EmptyAccessibleName(node_id) => write!(
                formatter,
                "accessible name must not be empty for node {}",
                node_id.as_str()
            ),
            Self::EmptyAccessibleLabel(node_id) => write!(
                formatter,
                "accessible label must not be empty for node {}",
                node_id.as_str()
            ),
            Self::RoleMismatch {
                node_id,
                expected,
                actual,
            } => write!(
                formatter,
                "node {} has role {actual:?}; expected {expected:?}",
                node_id.as_str()
            ),
            Self::EmptyButtonLabel(node_id) => {
                write!(
                    formatter,
                    "button label must not be empty for node {}",
                    node_id.as_str()
                )
            }
            Self::EmptyActionId(node_id) => {
                write!(
                    formatter,
                    "action ID must not be empty for node {}",
                    node_id.as_str()
                )
            }
            Self::InvalidLogicalSize(node_id) => write!(
                formatter,
                "logical dimensions must be positive for node {}",
                node_id.as_str()
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

impl SemanticTree {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut node_ids = HashSet::new();
        let mut errors = Vec::new();
        validate_node(&self.root, &mut node_ids, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_node(
    node: &SemanticNode,
    node_ids: &mut HashSet<NodeId>,
    errors: &mut Vec<ValidationError>,
) {
    if node.id.as_str().trim().is_empty() {
        errors.push(ValidationError::EmptyNodeId);
    } else if !node_ids.insert(node.id.clone()) {
        errors.push(ValidationError::DuplicateNodeId(node.id.clone()));
    }

    if node
        .size
        .is_some_and(|size| size.width == 0 || size.height == 0)
    {
        errors.push(ValidationError::InvalidLogicalSize(node.id.clone()));
    }

    if node.accessibility.name.trim().is_empty() {
        errors.push(ValidationError::EmptyAccessibleName(node.id.clone()));
    }
    if node
        .accessibility
        .label
        .as_ref()
        .is_some_and(|label| label.trim().is_empty())
    {
        errors.push(ValidationError::EmptyAccessibleLabel(node.id.clone()));
    }

    let expected_role = match &node.kind {
        NodeKind::Container { .. } => SemanticRole::Container,
        NodeKind::Text { .. } => SemanticRole::Text,
        NodeKind::Button { .. } => SemanticRole::Button,
    };
    if node.accessibility.role != expected_role {
        errors.push(ValidationError::RoleMismatch {
            node_id: node.id.clone(),
            expected: expected_role,
            actual: node.accessibility.role,
        });
    }

    match &node.kind {
        NodeKind::Container { children, .. } => {
            for child in children {
                validate_node(child, node_ids, errors);
            }
        }
        NodeKind::Text { .. } => {}
        NodeKind::Button { label, action } => {
            if label.trim().is_empty() {
                errors.push(ValidationError::EmptyButtonLabel(node.id.clone()));
            }
            if action.as_str().trim().is_empty() {
                errors.push(ValidationError::EmptyActionId(node.id.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Accessibility, ActionId, Layout, LogicalSize, NodeId, NodeKind, SemanticNode, SemanticRole,
        SemanticTree, ValidationError,
    };

    fn container(id: &str, children: Vec<SemanticNode>) -> SemanticNode {
        SemanticNode {
            id: NodeId::new(id),
            accessibility: Accessibility {
                role: SemanticRole::Container,
                name: "Content".to_owned(),
                label: None,
            },
            size: Some(LogicalSize {
                width: 480,
                height: 320,
            }),
            kind: NodeKind::Container {
                layout: Layout::Vertical,
                children,
            },
        }
    }

    fn button(id: &str, action: &str) -> SemanticNode {
        SemanticNode {
            id: NodeId::new(id),
            accessibility: Accessibility {
                role: SemanticRole::Button,
                name: "Activate".to_owned(),
                label: Some("Activate action".to_owned()),
            },
            size: None,
            kind: NodeKind::Button {
                label: "Activate".to_owned(),
                action: ActionId::new(action),
            },
        }
    }

    #[test]
    fn accepts_nested_content_with_unique_ids_and_accessible_actions() {
        let tree = SemanticTree {
            root: container(
                "root",
                vec![
                    SemanticNode {
                        id: NodeId::new("heading"),
                        accessibility: Accessibility {
                            role: SemanticRole::Text,
                            name: "Status".to_owned(),
                            label: None,
                        },
                        size: None,
                        kind: NodeKind::Text {
                            text: "Ready".to_owned(),
                        },
                    },
                    button("activate", "submit"),
                ],
            ),
        };

        assert_eq!(tree.validate(), Ok(()));
    }

    #[test]
    fn rejects_duplicate_node_ids_across_the_tree() {
        let tree = SemanticTree {
            root: container("root", vec![button("root", "submit")]),
        };

        assert_eq!(
            tree.validate(),
            Err(vec![ValidationError::DuplicateNodeId(NodeId::new("root"))])
        );
    }

    #[test]
    fn rejects_blank_accessible_metadata_and_button_labels() {
        let mut node = button("activate", "submit");
        node.accessibility.name = " \t".to_owned();
        node.accessibility.label = Some(" ".to_owned());
        if let NodeKind::Button { label, .. } = &mut node.kind {
            *label = "\n".to_owned();
        }
        let tree = SemanticTree {
            root: container("root", vec![node]),
        };

        let errors = tree.validate().unwrap_err();
        assert!(
            errors.contains(&ValidationError::EmptyAccessibleName(NodeId::new(
                "activate"
            )))
        );
        assert!(
            errors.contains(&ValidationError::EmptyAccessibleLabel(NodeId::new(
                "activate"
            )))
        );
        assert!(errors.contains(&ValidationError::EmptyButtonLabel(NodeId::new("activate"))));
    }

    #[test]
    fn rejects_role_mismatches_and_empty_action_ids() {
        let mut node = button("activate", "submit");
        node.accessibility.role = SemanticRole::Text;
        if let NodeKind::Button { action, .. } = &mut node.kind {
            *action = ActionId::new(" ");
        }
        let tree = SemanticTree {
            root: container("root", vec![node]),
        };

        let errors = tree.validate().unwrap_err();
        assert!(errors.contains(&ValidationError::RoleMismatch {
            node_id: NodeId::new("activate"),
            expected: SemanticRole::Button,
            actual: SemanticRole::Text,
        }));
        assert!(errors.contains(&ValidationError::EmptyActionId(NodeId::new("activate"))));
    }

    #[test]
    fn rejects_empty_node_ids_and_nonpositive_logical_dimensions() {
        let mut node = button(" ", "submit");
        node.size = Some(LogicalSize {
            width: 0,
            height: 24,
        });
        let tree = SemanticTree {
            root: container("root", vec![node]),
        };

        let errors = tree.validate().unwrap_err();
        assert!(errors.contains(&ValidationError::EmptyNodeId));
        assert!(errors.contains(&ValidationError::InvalidLogicalSize(NodeId::new(" "))));
    }
}
