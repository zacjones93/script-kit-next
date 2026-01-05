🧩 Packing 4 file(s)...
📝 Files selected:
  • src/app_layout.rs
  • src/app_render.rs
  • src/app_actions.rs
  • src/app_navigation.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 4
</notes>
</file_summary>

<directory_structure>
src/app_layout.rs
src/app_render.rs
src/app_actions.rs
src/app_navigation.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/app_layout.rs">
// Layout calculation methods for ScriptListApp
// This file is included via include!() macro in main.rs
// Contains: build_component_bounds, build_layout_info

impl ScriptListApp {
    /// Build component bounds for the debug grid overlay based on current view
    ///
    /// This creates approximate bounds for major UI components based on our
    /// known layout structure. Since GPUI doesn't expose layout bounds at runtime,
    /// we calculate them based on our layout constants.
    fn build_component_bounds(
        &self,
        window_size: gpui::Size<gpui::Pixels>,
    ) -> Vec<debug_grid::ComponentBounds> {
        use debug_grid::{BoxModel, ComponentBounds, ComponentType};

        let mut bounds = Vec::new();
        let width = window_size.width;
        let height = window_size.height;

        // Layout constants from panel.rs and list_item.rs
        // Header: py(HEADER_PADDING_Y=8) + max(input=22px, buttons=28px) + py(8) + divider(1px)
        // The buttons are 28px tall, input is 22px, so header content height is 28px
        // Total: 8 + 28 + 8 + 1 = 45px
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = px(HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT); // 45px

        // Content padding matches HEADER_PADDING_X
        let content_padding = HEADER_PADDING_X;

        // Main content area (below header)
        let content_top = header_height;
        let content_height = height - header_height;

        // Determine the current view type and build appropriate bounds
        let view_name = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
            AppView::AppLauncherView { .. } => "AppLauncher",
            AppView::WindowSwitcherView { .. } => "WindowSwitcher",
            AppView::DesignGalleryView { .. } => "DesignGallery",
            AppView::ActionsDialog => "ActionsDialog",
        };

        // Header bounds (includes padding + input + divider) - common to all views
        bounds.push(
            ComponentBounds::new(
                "Header",
                gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: gpui::size(width, header_height),
                },
            )
            .with_type(ComponentType::Header)
            .with_padding(BoxModel::symmetric(HEADER_PADDING_Y, content_padding)),
        );

        // Build view-specific bounds
        match &self.current_view {
            AppView::ScriptList => {
                // ScriptList has left panel (50%) + right preview panel (50%)
                let list_width = width * 0.5;
                let item_height = px(48.0);

                bounds.push(
                    ComponentBounds::new(
                        "ScriptList",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(list_width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                // Add sample list items
                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * 48.0);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("ListItem[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(list_width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding))
                        .with_margin(BoxModel::uniform(0.0)),
                    );
                }

                // Preview panel (right side)
                bounds.push(
                    ComponentBounds::new(
                        "PreviewPanel",
                        gpui::Bounds {
                            origin: gpui::point(list_width, content_top),
                            size: gpui::size(width - list_width, content_height),
                        },
                    )
                    .with_type(ComponentType::Container)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::DivPrompt { .. } => {
                // DivPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "DivContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::EditorPrompt { .. } => {
                // EditorPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "EditorContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::TermPrompt { .. } => {
                // TermPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "TerminalContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::ArgPrompt { choices, .. } => {
                // ArgPrompt may have choices list
                if choices.is_empty() {
                    // No choices - just input area
                    bounds.push(
                        ComponentBounds::new(
                            "ArgInput",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::Prompt)
                        .with_padding(BoxModel::uniform(content_padding)),
                    );
                } else {
                    // Has choices - show list
                    let item_height = px(48.0);
                    bounds.push(
                        ComponentBounds::new(
                            "ChoicesList",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::List)
                        .with_padding(BoxModel::uniform(0.0)),
                    );

                    // Add choice items
                    for i in 0..choices.len().min(5) {
                        let item_top = content_top + px(i as f32 * 48.0);
                        if item_top + item_height > height {
                            break;
                        }
                        bounds.push(
                            ComponentBounds::new(
                                format!("Choice[{}]", i),
                                gpui::Bounds {
                                    origin: gpui::point(px(0.), item_top),
                                    size: gpui::size(width, item_height),
                                },
                            )
                            .with_type(ComponentType::ListItem)
                            .with_padding(BoxModel::symmetric(12.0, content_padding)),
                        );
                    }
                }
            }

            AppView::FormPrompt { .. } => {
                bounds.push(
                    ComponentBounds::new(
                        "FormContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::SelectPrompt { .. } | AppView::PathPrompt { .. } => {
                // List-based prompts
                let item_height = px(48.0);
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * 48.0);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("Item[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding)),
                    );
                }
            }

            // Other prompts - generic full-width content
            _ => {
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }
        }

        // Only add header detail bounds for ScriptList view (the original behavior)
        if matches!(self.current_view, AppView::ScriptList) {
            let list_width = width * 0.5;

            // Input field in header
            // Positioned at: px(HEADER_PADDING_X) = 16, py(HEADER_PADDING_Y) = 8
            // The input is vertically centered in the header (which has 28px content height)
            // Input height is ~22px (CURSOR_HEIGHT_LG=18 + CURSOR_MARGIN_Y*2=4)
            const INPUT_HEIGHT: f32 = 22.0;
            let input_x = px(content_padding);
            let input_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0); // Vertically centered
                                                                                       // Input takes flex-1, estimate it takes most of the header width before buttons
                                                                                       // Buttons area is roughly: Run(50) + divider(20) + Actions(70) + divider(20) + Logo(16) + padding(16) = ~192px
            let buttons_area_width = px(200.);
            let input_width = width - px(content_padding) - buttons_area_width;

            bounds.push(
                ComponentBounds::new(
                    "SearchInput",
                    gpui::Bounds {
                        origin: gpui::point(input_x, input_y),
                        size: gpui::size(input_width, px(INPUT_HEIGHT)),
                    },
                )
                .with_type(ComponentType::Input)
                .with_padding(BoxModel::symmetric(0.0, 0.0)),
            );

            // Header buttons (right side)
            // Buttons are h(28px) positioned at top of content area (after top padding)
            let button_height = px(BUTTON_HEIGHT);
            let button_y = px(HEADER_PADDING_Y); // Buttons at top of content area

            // Buttons layout from right to left:
            // [SearchInput flex-1] [Run ~45px] [|] [Actions ~70px] [|] [Logo 16px] [padding 16px]
            // Spacing: gap=12, divider ~8px each side = ~20px between groups
            let logo_size = px(16.);
            let right_padding = px(content_padding);

            // Logo (Script Kit icon) - rightmost, 16x16 vertically centered in button area
            let logo_x = width - right_padding - logo_size;
            let logo_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - 16.0) / 2.0); // Vertically centered
            bounds.push(
                ComponentBounds::new(
                    "Lg", // Short name for Logo to fit in small space
                    gpui::Bounds {
                        origin: gpui::point(logo_x, logo_y),
                        size: gpui::size(logo_size, logo_size),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(0.0)),
            );

            // Actions button - left of divider, left of logo
            // Actual button text "Actions ⌘K" is roughly 80-90px wide
            let actions_width = px(85.);
            let actions_x = logo_x - px(24.) - actions_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Actions", // Shortened from ActionsButton
                    gpui::Bounds {
                        origin: gpui::point(actions_x, button_y),
                        size: gpui::size(actions_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

            // Run button - left of divider, left of Actions
            // Actual button text "Run ↵" is roughly 50-60px wide
            let run_width = px(55.);
            let run_x = actions_x - px(24.) - run_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Run", // Shortened from RunButton
                    gpui::Bounds {
                        origin: gpui::point(run_x, button_y),
                        size: gpui::size(run_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

            // Preview panel contents (right 50% of window)
            // Preview has its own padding, content starts at list_width + padding
            let preview_padding = 16.0_f32;
            let preview_left = list_width + px(preview_padding);
            let preview_width = width * 0.5 - px(preview_padding * 2.0);

            // Script path label (small text at top of preview)
            bounds.push(
                ComponentBounds::new(
                    "ScriptPath",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(8.)),
                        size: gpui::size(preview_width, px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::symmetric(2.0, 0.0)),
            );

            // Script title (large heading)
            bounds.push(
                ComponentBounds::new(
                    "ScriptTitle",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(32.)),
                        size: gpui::size(preview_width, px(32.)),
                    },
                )
                .with_type(ComponentType::Header)
                .with_padding(BoxModel::symmetric(4.0, 0.0)),
            );

            // Description label
            bounds.push(
                ComponentBounds::new(
                    "DescLabel", // Shortened
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(72.)),
                        size: gpui::size(px(80.), px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(2.0)),
            );

            // Description value
            bounds.push(
                ComponentBounds::new(
                    "DescValue", // Shortened
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(92.)),
                        size: gpui::size(preview_width, px(20.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::symmetric(2.0, 0.0)),
            );

            // Code Preview label
            bounds.push(
                ComponentBounds::new(
                    "CodeLabel", // Shortened from CodePreviewLabel
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(130.)),
                        size: gpui::size(px(100.), px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(2.0)),
            );

            // Code preview area
            bounds.push(
                ComponentBounds::new(
                    "CodePreview",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(150.)),
                        size: gpui::size(preview_width, height - content_top - px(170.)),
                    },
                )
                .with_type(ComponentType::Container)
                .with_padding(BoxModel::uniform(12.0)),
            );

            // List item icons (left side of each list item)
            // Icons are typically 24x24, positioned with some padding from left edge
            // Item height is 48px, icon vertically centered: (48 - 24) / 2 = 12px from top
            let item_height = px(48.0);
            for i in 0..5 {
                let item_top = content_top + px(i as f32 * 48.0);
                if item_top + item_height > height {
                    break;
                }
                bounds.push(
                    ComponentBounds::new(
                        format!("Icon[{}]", i),
                        gpui::Bounds {
                            origin: gpui::point(px(content_padding), item_top + px(12.)),
                            size: gpui::size(px(24.), px(24.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
            }
        } // End of ScriptList-specific bounds

        bounds
    }

    /// Build complete layout info for the getLayoutInfo() SDK function.
    ///
    /// This provides AI agents with detailed component information including:
    /// - Bounds (position and size)
    /// - Box model (padding, margin, gap)
    /// - Flex properties (direction, grow, align)
    /// - Human-readable explanations of why components are sized as they are
    ///
    /// This is the "why" function - it explains the layout, not just shows it.
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // TODO: Get actual window size once we have access to window in this context
        // For now, use default values
        let window_width = 750.0_f32;
        let window_height = 500.0_f32;

        // Determine current prompt type
        let prompt_type = match &self.current_view {
            AppView::ScriptList => "mainMenu",
            AppView::ArgPrompt { .. } => "arg",
            AppView::DivPrompt { .. } => "div",
            AppView::FormPrompt { .. } => "form",
            AppView::TermPrompt { .. } => "term",
            AppView::EditorPrompt { .. } => "editor",
            AppView::SelectPrompt { .. } => "select",
            AppView::PathPrompt { .. } => "path",
            AppView::EnvPrompt { .. } => "env",
            AppView::DropPrompt { .. } => "drop",
            AppView::TemplatePrompt { .. } => "template",
            AppView::ClipboardHistoryView { .. } => "clipboardHistory",
            AppView::AppLauncherView { .. } => "appLauncher",
            AppView::WindowSwitcherView { .. } => "windowSwitcher",
            AppView::DesignGalleryView { .. } => "designGallery",
            AppView::ActionsDialog => "actionsDialog",
        };

        let mut components = Vec::new();

        // Layout constants (same as build_component_bounds)
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT; // 45px
        let list_width = window_width * 0.5;
        let content_top = header_height;
        let content_height = window_height - header_height;

        // Root container
        components.push(
            LayoutComponentInfo::new("Window", LayoutComponentType::Container)
                .with_bounds(0.0, 0.0, window_width, window_height)
                .with_flex_column()
                .with_depth(0)
                .with_explanation("Root window container. Uses flex-column layout."),
        );

        // Header
        components.push(
            LayoutComponentInfo::new("Header", LayoutComponentType::Header)
                .with_bounds(0.0, 0.0, window_width, header_height)
                .with_padding(HEADER_PADDING_Y, HEADER_PADDING_X, HEADER_PADDING_Y, HEADER_PADDING_X)
                .with_flex_row()
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(format!(
                    "Height = padding({}) + content({}) + padding({}) + divider({}) = {}px. Uses flex-row with items-center.",
                    HEADER_PADDING_Y, BUTTON_HEIGHT, HEADER_PADDING_Y, DIVIDER_HEIGHT, header_height
                )),
        );

        // Search input in header
        const INPUT_HEIGHT: f32 = 22.0;
        let input_y = HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0;
        let buttons_area_width = 200.0;
        let input_width = window_width - HEADER_PADDING_X - buttons_area_width;

        components.push(
            LayoutComponentInfo::new("SearchInput", LayoutComponentType::Input)
                .with_bounds(HEADER_PADDING_X, input_y, input_width, INPUT_HEIGHT)
                .with_flex_grow(1.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "flex-grow:1 fills remaining space. Width = window({}) - padding({}) - buttons_area({}) = {}px. Vertically centered in header.",
                    window_width, HEADER_PADDING_X, buttons_area_width, input_width
                )),
        );

        // Content area
        components.push(
            LayoutComponentInfo::new("ContentArea", LayoutComponentType::Container)
                .with_bounds(0.0, content_top, window_width, content_height)
                .with_flex_row()
                .with_flex_grow(1.0)
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(
                    "flex-grow:1 fills remaining height after header. Uses flex-row to create side-by-side panels.".to_string()
                ),
        );

        // Script list (left panel) - 50% width
        components.push(
            LayoutComponentInfo::new("ScriptList", LayoutComponentType::List)
                .with_bounds(0.0, content_top, list_width, content_height)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = 50% of window = {}px. Uses uniform_list for virtualized scrolling with 48px item height.",
                    list_width
                )),
        );

        // Preview panel (right panel) - remaining 50%
        let preview_width = window_width - list_width;
        components.push(
            LayoutComponentInfo::new("PreviewPanel", LayoutComponentType::Panel)
                .with_bounds(list_width, content_top, preview_width, content_height)
                .with_padding(16.0, 16.0, 16.0, 16.0)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = remaining 50% = {}px. Has 16px padding on all sides.",
                    preview_width
                )),
        );

        // List items (sample of first few visible)
        const LIST_ITEM_HEIGHT: f32 = 48.0;
        let visible_items = ((content_height / LIST_ITEM_HEIGHT) as usize).min(5);
        for i in 0..visible_items {
            let item_top = content_top + (i as f32 * LIST_ITEM_HEIGHT);
            components.push(
                LayoutComponentInfo::new(format!("ListItem[{}]", i), LayoutComponentType::ListItem)
                    .with_bounds(0.0, item_top, list_width, LIST_ITEM_HEIGHT)
                    .with_padding(12.0, 16.0, 12.0, 16.0)
                    .with_gap(8.0)
                    .with_flex_row()
                    .with_depth(3)
                    .with_parent("ScriptList")
                    .with_explanation(format!(
                        "Fixed height = {}px. Uses flex-row with gap:8px for icon + text layout. Padding: 12px vertical, 16px horizontal.",
                        LIST_ITEM_HEIGHT
                    )),
            );
        }

        // Button group in header
        let button_y = HEADER_PADDING_Y;
        let button_height = BUTTON_HEIGHT;

        // Logo button (rightmost)
        let logo_x = window_width - HEADER_PADDING_X - 20.0;
        components.push(
            LayoutComponentInfo::new("LogoButton", LayoutComponentType::Button)
                .with_bounds(logo_x, button_y, 20.0, button_height)
                .with_padding(4.0, 4.0, 4.0, 4.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation("Fixed 20px width. Positioned at right edge with 16px margin."),
        );

        // Actions button
        let actions_width = 85.0;
        let actions_x = logo_x - 24.0 - actions_width;
        components.push(
            LayoutComponentInfo::new("ActionsButton", LayoutComponentType::Button)
                .with_bounds(actions_x, button_y, actions_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of logo with 24px spacing (includes divider).",
                    actions_width
                )),
        );

        // Run button
        let run_width = 55.0;
        let run_x = actions_x - 24.0 - run_width;
        components.push(
            LayoutComponentInfo::new("RunButton", LayoutComponentType::Button)
                .with_bounds(run_x, button_y, run_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of Actions with 24px spacing.",
                    run_width
                )),
        );

        LayoutInfo {
            window_width,
            window_height,
            prompt_type: prompt_type.to_string(),
            components,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

</file>

<file path="src/app_render.rs">
impl ScriptListApp {
    /// Read the first N lines of a script file for preview
    #[allow(dead_code)]
    fn read_script_preview(path: &std::path::Path, max_lines: usize) -> String {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let preview: String = content
                    .lines()
                    .take(max_lines)
                    .collect::<Vec<_>>()
                    .join("\n");
                logging::log(
                    "UI",
                    &format!(
                        "Preview loaded: {} ({} lines read)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        content.lines().count().min(max_lines)
                    ),
                );
                preview
            }
            Err(e) => {
                logging::log("UI", &format!("Preview error: {} - {}", path.display(), e));
                format!("Error reading file: {}", e)
            }
        }
    }

    // NOTE: render_toasts() removed - now using gpui-component's NotificationList
    // via the Root wrapper. Toasts are flushed via flush_pending_toasts() in render().
    // See toast_manager.rs for the queue and main.rs for the flush logic.

    /// Render the preview panel showing details of the selected script/scriptlet
    fn render_preview_panel(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Get grouped results to map from selected_index to actual result (cached)
        // Clone to avoid borrow issues with self.selected_index access
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let selected_result = match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        };

        // Use design tokens for GLOBAL theming - design applies to ALL components
        let tokens = get_tokens(self.current_design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();

        // Map design tokens to local variables (all designs use tokens now)
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;
        let bg_search_box = colors.background_tertiary;
        let border_radius = visual.radius_md;
        let font_family = typography.font_family;

        // Preview panel container with left border separator
        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgb(bg_main))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(font_family);

        // P4: Compute match indices lazily for visible preview (only one result at a time)
        let computed_filter = self.computed_filter_text.clone();

        match selected_result {
            Some(ref result) => {
                // P4: Lazy match indices computation for preview panel
                let match_indices =
                    scripts::compute_match_indices_for_result(result, &computed_filter);

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        let script = &script_match.script;

                        // Source indicator with match highlighting (e.g., "script: foo.ts")
                        let filename = &script_match.filename;
                        // P4: Use lazily computed indices instead of stored (empty) ones
                        let filename_indices = &match_indices.filename_indices;

                        // Render filename with highlighted matched characters
                        let path_segments =
                            render_path_with_highlights(filename, filename, filename_indices);
                        let accent_color = colors.accent;

                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("script: "),
                            );

                        for (text, is_highlighted) in path_segments {
                            let color = if is_highlighted {
                                rgb(accent_color)
                            } else {
                                rgba((text_muted << 8) | 0x99)
                            };
                            path_div = path_div.child(div().text_color(color).child(text));
                        }

                        panel = panel.child(path_div);

                        // Script name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(format!("{}.{}", script.name, script.extension)),
                        );

                        // Description (if present)
                        if let Some(desc) = &script.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Code preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Code Preview"),
                        );

                        // Use cached syntax-highlighted lines (avoids file I/O and highlighting on every render)
                        let script_path = script.path.to_string_lossy().to_string();
                        let lang = script.extension.clone();
                        let lines = self
                            .get_or_update_preview_cache(&script_path, &lang)
                            .to_vec();

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        let scriptlet = &scriptlet_match.scriptlet;

                        // Source indicator with match highlighting (e.g., "scriptlet: foo.md")
                        if let Some(ref display_file_path) = scriptlet_match.display_file_path {
                            // P4: Use lazily computed indices instead of stored (empty) ones
                            let filename_indices = &match_indices.filename_indices;

                            // Render filename with highlighted matched characters
                            let path_segments = render_path_with_highlights(
                                display_file_path,
                                display_file_path,
                                filename_indices,
                            );
                            let accent_color = colors.accent;

                            let mut path_div = div()
                                .flex()
                                .flex_row()
                                .text_xs()
                                .font_family(typography.font_family_mono)
                                .pb(px(spacing.padding_xs))
                                .overflow_x_hidden()
                                .child(
                                    div()
                                        .text_color(rgba((text_muted << 8) | 0x99))
                                        .child("scriptlet: "),
                                );

                            for (text, is_highlighted) in path_segments {
                                let color = if is_highlighted {
                                    rgb(accent_color)
                                } else {
                                    rgba((text_muted << 8) | 0x99)
                                };
                                path_div = path_div.child(div().text_color(color).child(text));
                            }

                            panel = panel.child(path_div);
                        }

                        // Scriptlet name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(scriptlet.name.clone()),
                        );

                        // Description (if present)
                        if let Some(desc) = &scriptlet.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Shortcut (if present)
                        if let Some(shortcut) = &scriptlet.shortcut {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Hotkey"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(shortcut.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Content Preview"),
                        );

                        // Display scriptlet code with syntax highlighting (first 15 lines)
                        // Note: Scriptlets store code in memory, no file I/O needed (no cache benefit)
                        let code_preview: String = scriptlet
                            .code
                            .lines()
                            .take(15)
                            .collect::<Vec<_>>()
                            .join("\n");

                        // Determine language from tool (bash, js, etc.)
                        let lang = match scriptlet.tool.as_str() {
                            "bash" | "zsh" | "sh" => "bash",
                            "node" | "bun" => "js",
                            _ => &scriptlet.tool,
                        };
                        let lines = highlight_code_lines(&code_preview, lang);

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        let builtin = &builtin_match.entry;

                        // Built-in name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(builtin.name.clone()),
                        );

                        // Description
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Description"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(builtin.description.clone()),
                                ),
                        );

                        // Keywords
                        if !builtin.keywords.is_empty() {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keywords"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(builtin.keywords.join(", ")),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Feature type indicator
                        let feature_type: String = match &builtin.feature {
                            builtins::BuiltInFeature::ClipboardHistory => {
                                "Clipboard History Manager".to_string()
                            }
                            builtins::BuiltInFeature::AppLauncher => {
                                "Application Launcher".to_string()
                            }
                            builtins::BuiltInFeature::App(name) => name.clone(),
                            builtins::BuiltInFeature::WindowSwitcher => {
                                "Window Manager".to_string()
                            }
                            builtins::BuiltInFeature::DesignGallery => "Design Gallery".to_string(),
                            builtins::BuiltInFeature::AiChat => "AI Assistant".to_string(),
                            builtins::BuiltInFeature::Notes => "Notes & Scratchpad".to_string(),
                            builtins::BuiltInFeature::MenuBarAction(_) => {
                                "Menu Bar Action".to_string()
                            }
                            builtins::BuiltInFeature::SystemAction(_) => {
                                "System Action".to_string()
                            }
                            builtins::BuiltInFeature::WindowAction(_) => {
                                "Window Action".to_string()
                            }
                            builtins::BuiltInFeature::NotesCommand(_) => {
                                "Notes Command".to_string()
                            }
                            builtins::BuiltInFeature::AiCommand(_) => "AI Command".to_string(),
                            builtins::BuiltInFeature::ScriptCommand(_) => {
                                "Script Creation".to_string()
                            }
                            builtins::BuiltInFeature::PermissionCommand(_) => {
                                "Permission Management".to_string()
                            }
                            builtins::BuiltInFeature::FrecencyCommand(_) => {
                                "Suggested Items".to_string()
                            }
                        };
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Feature Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(feature_type),
                                ),
                        );
                    }
                    scripts::SearchResult::App(app_match) => {
                        let app = &app_match.app;

                        // App name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(app.name.clone()),
                        );

                        // Path
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Path"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(app.path.to_string_lossy().to_string()),
                                ),
                        );

                        // Bundle ID (if available)
                        if let Some(bundle_id) = &app.bundle_id {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Bundle ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(bundle_id.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Application"),
                                ),
                        );
                    }
                    scripts::SearchResult::Window(window_match) => {
                        let window = &window_match.window;

                        // Window title header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(window.title.clone()),
                        );

                        // App name
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Application"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(window.app.clone()),
                                ),
                        );

                        // Bounds
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Position & Size"),
                                )
                                .child(div().text_sm().text_color(rgb(text_secondary)).child(
                                    format!(
                                        "{}×{} at ({}, {})",
                                        window.bounds.width,
                                        window.bounds.height,
                                        window.bounds.x,
                                        window.bounds.y
                                    ),
                                )),
                        );

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Window"),
                                ),
                        );
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        let agent = &agent_match.agent;

                        // Source indicator with agent path
                        let filename = agent
                            .path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "agent".to_string());

                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("agent: "),
                            );

                        path_div = path_div.child(
                            div()
                                .text_color(rgba((text_muted << 8) | 0x99))
                                .child(filename),
                        );

                        panel = panel.child(path_div);

                        // Agent name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(agent.name.clone()),
                        );

                        // Description
                        if let Some(desc) = &agent.description {
                            panel = panel.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_secondary))
                                    .pb(px(spacing.padding_md))
                                    .child(desc.clone()),
                            );
                        }

                        // Backend info
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Backend"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{:?}", agent.backend)),
                                ),
                        );

                        // Kit info if available
                        if let Some(kit) = &agent.kit {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Kit"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(kit.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Agent"),
                                ),
                        );
                    }
                }
            }
            None => {
                logging::log("UI", "Preview panel: No selection");
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child(
                            if self.filter_text.is_empty()
                                && self.scripts.is_empty()
                                && self.scriptlets.is_empty()
                            {
                                "No scripts or snippets found"
                            } else if !self.filter_text.is_empty() {
                                "No matching scripts"
                            } else {
                                "Select a script to preview"
                            },
                        ),
                );
            }
        }

        panel
    }

    /// Get the ScriptInfo for the currently focused/selected script
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                match result {
                    scripts::SearchResult::Script(m) => Some(ScriptInfo::new(
                        &m.script.name,
                        m.script.path.to_string_lossy(),
                    )),
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets don't have a path, use name as identifier
                        Some(ScriptInfo::new(
                            &m.scriptlet.name,
                            format!("scriptlet:{}", &m.scriptlet.name),
                        ))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        Some(ScriptInfo::new(
                            &m.entry.name,
                            format!("builtin:{}", &m.entry.id),
                        ))
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        Some(ScriptInfo::new(
                            &m.app.name,
                            m.app.path.to_string_lossy().to_string(),
                        ))
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        Some(ScriptInfo::new(
                            &m.window.title,
                            format!("window:{}", m.window.id),
                        ))
                    }
                    scripts::SearchResult::Agent(m) => {
                        // Agents use their path as identifier
                        Some(ScriptInfo::new(
                            &m.agent.name,
                            format!("agent:{}", m.agent.path.to_string_lossy()),
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for actions dialog
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC closes window from ActionsDialog too)
                // ActionsDialog has no other key handling, so we just call the global handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // Simple actions dialog stub with design tokens
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(div().text_lg().child("Actions (Cmd+K)"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .mt(px(design_spacing.margin_md))
                    .child("• Create script\n• Edit script\n• Reload\n• Settings\n• Quit"),
            )
            .child(
                div()
                    .mt(px(design_spacing.margin_lg))
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}

/// Helper function to render a group header style item with actual visual styling
fn render_group_header_item(
    ix: usize,
    is_selected: bool,
    style: &designs::group_header_variations::GroupHeaderStyle,
    spacing: &designs::DesignSpacing,
    typography: &designs::DesignTypography,
    visual: &designs::DesignVisual,
    colors: &designs::DesignColors,
) -> AnyElement {
    use designs::group_header_variations::GroupHeaderStyle;

    let name_owned = style.name().to_string();
    let desc_owned = style.description().to_string();

    let mut item_div = div()
        .id(ElementId::NamedInteger("gallery-header".into(), ix as u64))
        .w_full()
        .h(px(LIST_ITEM_HEIGHT))
        .px(px(spacing.padding_lg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.gap_md));

    if is_selected {
        item_div = item_div.bg(rgb(colors.background_selected));
    }

    // Create the preview element based on the style
    let preview = match style {
        // Text Only styles - vary font weight and style
        GroupHeaderStyle::UppercaseLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::UppercaseCenter => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::SmallCapsLeft => {
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(colors.text_secondary))
                .child("MAIN") // Would use font-variant: small-caps if available
        }
        GroupHeaderStyle::BoldLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::LightLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::LIGHT)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),
        GroupHeaderStyle::MonospaceLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_family(typography.font_family_mono)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),

        // With Lines styles
        GroupHeaderStyle::LineLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().w(px(24.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::LineRight => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBothSides => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBelow => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineAbove => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DoubleLine => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap(px(1.0))
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border))),

        // With Background styles
        GroupHeaderStyle::PillBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(10.0))
                    .bg(rgba((colors.accent << 8) | 0x30))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::FullWidthBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.accent << 8) | 0x20))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::SubtleBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x90))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::GradientFade => {
            // Simulated with opacity fade
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .child(
                    div()
                        .px(px(16.0))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_secondary))
                        .child("~  MAIN  ~"),
                )
        }
        GroupHeaderStyle::BorderedBox => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),

        // Minimal styles
        GroupHeaderStyle::DotPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(rgb(colors.text_muted)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DashPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("- MAIN"),
        GroupHeaderStyle::BulletPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(6.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(rgb(colors.accent)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::ArrowPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{25B8} MAIN"),
        GroupHeaderStyle::ChevronPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{203A} MAIN"),
        GroupHeaderStyle::Dimmed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .opacity(0.5)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),

        // Decorative styles
        GroupHeaderStyle::Bracketed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("[MAIN]"),
        GroupHeaderStyle::Quoted => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\"MAIN\""),
        GroupHeaderStyle::Tagged => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(1.0))
                    .bg(rgba((colors.accent << 8) | 0x40))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::Numbered => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(colors.accent))
                    .child("01."),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::IconPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .bg(rgb(colors.accent))
                    .rounded(px(1.0)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
    };

    item_div
        // Preview element
        .child(preview)
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child(name_owned),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(desc_owned),
                ),
        )
        .into_any_element()
}

</file>

<file path="src/app_actions.rs">
// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

impl ScriptListApp {
    /// Handle action selection from the actions dialog
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
        logging::log("UI", &format!("Action selected: {}", action_id));

        // Close the dialog and return to script list
        self.current_view = AppView::ScriptList;

        match action_id.as_str() {
            "create_script" => {
                logging::log("UI", "Create script action - opening scripts folder");
                // Open ~/.scriptkit/scripts/ in Finder for now (future: create script dialog)
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.last_output = Some(SharedString::from("Opened scripts folder"));
                // Hide window after opening folder and set reset flag
                script_kit_gpui::set_main_window_visible(false);
                NEEDS_RESET.store(true, Ordering::SeqCst);
                cx.hide();
            }
            "run_script" => {
                logging::log("UI", "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                logging::log("UI", "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                logging::log("UI", "Reveal in Finder action");
                if let Some(result) = self.get_selected_result() {
                    match result {
                        scripts::SearchResult::Script(script_match) => {
                            let path_str = script_match.script.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed in Finder"));
                            // Hide window after revealing in Finder and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal scriptlets in Finder"));
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal built-in features"));
                        }
                        scripts::SearchResult::App(app_match) => {
                            let path_str = app_match.app.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed app in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal app in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed app in Finder"));
                            // Hide window after revealing app in Finder and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal windows in Finder"));
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            let path_str = agent_match.agent.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed agent in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal agent in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed agent in Finder"));
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(script_match) => {
                            Some(script_match.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::App(app_match) => {
                            Some(app_match.app.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy scriptlet path"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy built-in path"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot copy window path"));
                            None
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            Some(agent_match.agent.path.to_string_lossy().to_string())
                        }
                    };

                    if let Some(path_str) = path_opt {
                        // Use pbcopy on macOS for reliable clipboard access
                        #[cfg(target_os = "macos")]
                        {
                            use std::io::Write;
                            use std::process::{Command, Stdio};

                            match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                                Ok(mut child) => {
                                    if let Some(ref mut stdin) = child.stdin {
                                        if stdin.write_all(path_str.as_bytes()).is_ok() {
                                            let _ = child.wait();
                                            logging::log(
                                                "UI",
                                                &format!("Copied path to clipboard: {}", path_str),
                                            );
                                            self.last_output = Some(SharedString::from(format!(
                                                "Copied: {}",
                                                path_str
                                            )));
                                        } else {
                                            logging::log(
                                                "ERROR",
                                                "Failed to write to pbcopy stdin",
                                            );
                                            self.last_output =
                                                Some(SharedString::from("Failed to copy path"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to spawn pbcopy: {}", e),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }

                        // Fallback for non-macOS platforms
                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(&path_str) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!("Copied path to clipboard: {}", path_str),
                                        );
                                        self.last_output = Some(SharedString::from(format!(
                                            "Copied: {}",
                                            path_str
                                        )));
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "ERROR",
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.last_output =
                                            Some(SharedString::from("Failed to copy path"));
                                    }
                                },
                                Err(e) => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to access clipboard: {}", e),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to access clipboard"));
                                }
                            }
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "edit_script" => {
                logging::log("UI", "Edit script action");
                if let Some(result) = self.get_selected_result() {
                    match result {
                        scripts::SearchResult::Script(script_match) => {
                            self.edit_script(&script_match.script.path);
                            // Hide window after opening editor and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit scriptlets"));
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot edit built-in features"));
                        }
                        scripts::SearchResult::App(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit applications"));
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit windows"));
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            self.edit_script(&agent_match.agent.path);
                            // Hide window after opening editor and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No script selected"));
                }
            }
            "reload_scripts" => {
                logging::log("UI", "Reload scripts action");
                self.refresh_scripts(cx);
                self.last_output = Some(SharedString::from("Scripts reloaded"));
            }
            "settings" => {
                logging::log("UI", "Settings action");
                self.last_output = Some(SharedString::from("Settings (TODO)"));
            }
            "quit" => {
                logging::log("UI", "Quit action");
                // Clean up processes and PID file before quitting
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
            }
            "__cancel__" => {
                logging::log("UI", "Actions dialog cancelled");
            }
            _ => {
                // Check if this is an SDK action with has_action=true
                if let Some(ref actions) = self.sdk_actions {
                    if let Some(action) = actions.iter().find(|a| a.name == action_id) {
                        if action.has_action {
                            // Send ActionTriggered back to SDK
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action with handler: '{}' (has_action=true), sending ActionTriggered",
                                    action_id
                                ),
                            );
                            if let Some(ref sender) = self.response_sender {
                                let msg = protocol::Message::action_triggered(
                                    action_id.clone(),
                                    action.value.clone(),
                                    self.arg_input.text().to_string(),
                                );
                                if let Err(e) = sender.send(msg) {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to send ActionTriggered: {}", e),
                                    );
                                }
                            }
                        } else if let Some(ref value) = action.value {
                            // Submit value directly (has_action=false with value)
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action without handler: '{}' (has_action=false), submitting value: {:?}",
                                    action_id, value
                                ),
                            );
                            if let Some(ref sender) = self.response_sender {
                                let msg = protocol::Message::Submit {
                                    id: "action".to_string(),
                                    value: Some(value.clone()),
                                };
                                if let Err(e) = sender.send(msg) {
                                    logging::log("ERROR", &format!("Failed to send Submit: {}", e));
                                }
                            }
                        } else {
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action '{}' has no value and has_action=false",
                                    action_id
                                ),
                            );
                        }
                    } else {
                        logging::log("UI", &format!("Unknown action: {}", action_id));
                    }
                } else {
                    logging::log("UI", &format!("Unknown action: {}", action_id));
                }
            }
        }

        cx.notify();
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Triggering SDK action '{}' via shortcut (has_action={})",
                        action_name, action.has_action
                    ),
                );

                if action.has_action {
                    // Send ActionTriggered back to SDK
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::action_triggered(
                            action_name.to_string(),
                            action.value.clone(),
                            self.arg_input.text().to_string(),
                        );
                        if let Err(e) = sender.send(msg) {
                            logging::log(
                                "ERROR",
                                &format!("Failed to send ActionTriggered: {}", e),
                            );
                        }
                    }
                } else if let Some(ref value) = action.value {
                    // Submit value directly
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::Submit {
                            id: "action".to_string(),
                            value: Some(value.clone()),
                        };
                        if let Err(e) = sender.send(msg) {
                            logging::log("ERROR", &format!("Failed to send Submit: {}", e));
                        }
                    }
                }

                cx.notify();
                return true;
            }
        }
        false
    }
}

</file>

<file path="src/app_navigation.rs">
// App navigation methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: move_selection_up, move_selection_down, scroll_to_selected, etc.

impl ScriptListApp {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        // Find the first selectable (non-header) item index
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or before first selectable, can't go further up
        if let Some(first) = first_selectable {
            if self.selected_index <= first {
                // Already at the first selectable item, stay here
                return;
            }
        }

        if self.selected_index > 0 {
            let mut new_index = self.selected_index - 1;

            // Skip section headers when moving up
            while new_index > 0 {
                if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                    new_index -= 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at index 0
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let item_count = grouped_items.len();

        // Find the last selectable (non-header) item index
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or after last selectable, can't go further down
        if let Some(last) = last_selectable {
            if self.selected_index >= last {
                // Already at the last selectable item, stay here
                return;
            }
        }

        if self.selected_index < item_count.saturating_sub(1) {
            let mut new_index = self.selected_index + 1;

            // Skip section headers when moving down
            while new_index < item_count.saturating_sub(1) {
                if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                    new_index += 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at the end
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Scroll stabilization helper: only call scroll_to_reveal_item if we haven't already scrolled to this index.
    /// This prevents scroll jitter from redundant scroll calls.
    ///
    /// NOTE: Uses main_list_state (ListState) for the variable-height list() component,
    /// not the legacy list_scroll_handle (UniformListScrollHandle).
    fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
        let target = self.selected_index;

        // Check if we've already scrolled to this index
        if self.last_scrolled_index == Some(target) {
            return;
        }

        // Use perf guard for scroll timing
        let _scroll_perf = crate::perf::ScrollPerfGuard::new();

        // Perform the scroll using ListState for variable-height list
        // This scrolls the actual list() component used in render_script_list
        self.main_list_state.scroll_to_reveal_item(target);
        self.last_scrolled_index = Some(target);
    }

    /// Trigger scroll activity - shows the scrollbar and schedules fade-out
    ///
    /// This should be called whenever scroll-related activity occurs:
    /// - Keyboard up/down navigation
    /// - scroll_to_item calls
    /// - Mouse wheel scrolling (if tracked)
    fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
        self.is_scrolling = true;
        self.last_scroll_time = Some(std::time::Instant::now());

        // Schedule fade-out after 1000ms of inactivity
        cx.spawn(async move |this, cx| {
            Timer::after(std::time::Duration::from_millis(1000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    // Only hide if no new scroll activity occurred
                    if let Some(last_time) = app.last_scroll_time {
                        if last_time.elapsed() >= std::time::Duration::from_millis(1000) {
                            app.is_scrolling = false;
                            cx.notify();
                        }
                    }
                })
            });
        })
        .detach();

        cx.notify();
    }

    /// Apply a coalesced navigation delta in the given direction
    fn apply_nav_delta(&mut self, dir: NavDirection, delta: i32, cx: &mut Context<Self>) {
        let signed = match dir {
            NavDirection::Up => -delta,
            NavDirection::Down => delta,
        };
        self.move_selection_by(signed, cx);
    }

    /// Move selection by a signed delta (positive = down, negative = up)
    /// Used by NavCoalescer for batched movements
    ///
    /// IMPORTANT: This must use grouped results and skip section headers,
    /// just like move_selection_up/down. Otherwise, holding arrow keys
    /// can land on headers causing navigation to feel "stuck".
    fn move_selection_by(&mut self, delta: i32, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let len = grouped_items.len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }

        // Find bounds for selectable items (non-headers)
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If no selectable items, nothing to do
        let (first, last) = match (first_selectable, last_selectable) {
            (Some(f), Some(l)) => (f, l),
            _ => return,
        };

        // Calculate target index, clamping to valid range
        let target = (self.selected_index as i32 + delta).clamp(first as i32, last as i32) as usize;

        // If moving down (positive delta), skip headers forward
        // If moving up (negative delta), skip headers backward
        let new_index = if delta > 0 {
            // Moving down - find next non-header at or after target
            let mut idx = target;
            while idx <= last {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                idx += 1;
            }
            idx.min(last)
        } else if delta < 0 {
            // Moving up - find next non-header at or before target
            let mut idx = target;
            while idx >= first {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                if idx == 0 {
                    break;
                }
                idx -= 1;
            }
            idx.max(first)
        } else {
            // delta == 0, no movement
            self.selected_index
        };

        // Final validation: ensure we're not on a header
        if matches!(grouped_items.get(new_index), Some(GroupedListItem::SectionHeader(_))) {
            // Can't find a valid position, stay put
            return;
        }

        if new_index != self.selected_index {
            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("coalesced_nav");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Handle mouse wheel scroll events by converting to item-based scrolling.
    ///
    /// This bypasses GPUI's pixel-based scroll which has height calculation issues
    /// with variable-height items. Instead, we convert the scroll delta to item
    /// indices and use scroll_to_reveal_item() like keyboard navigation does.
    ///
    /// # Arguments
    /// * `delta_lines` - Scroll delta in "lines" (positive = scroll content up/view down)
    #[allow(dead_code)]
    pub fn handle_scroll_wheel(&mut self, delta_lines: f32, cx: &mut Context<Self>) {
        // Get current scroll position from ListState
        let current_item = self.main_list_state.logical_scroll_top().item_ix;

        // Get grouped results to find valid bounds
        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let max_item = grouped_items.len().saturating_sub(1);

        // Convert delta to items (negative delta = scroll down in content = move to higher indices)
        // Round to avoid tiny scrolls being ignored
        let items_to_scroll = (-delta_lines).round() as i32;

        // Calculate new target item, clamping to valid range
        let new_item = (current_item as i32 + items_to_scroll).clamp(0, max_item as i32) as usize;

        tracing::debug!(
            target: "SCROLL_STATE",
            delta_lines,
            current_item,
            new_item,
            items_to_scroll,
            "Mouse wheel scroll"
        );

        // Only scroll if we're moving to a different item
        if new_item != current_item {
            self.main_list_state.scroll_to_reveal_item(new_item);
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Ensure the navigation flush task is running. Spawns a background task
    /// that periodically flushes pending navigation deltas.
    fn ensure_nav_flush_task(&mut self, cx: &mut Context<Self>) {
        if self.nav_coalescer.flush_task_running {
            return;
        }
        self.nav_coalescer.flush_task_running = true;
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(NavCoalescer::WINDOW).await;
                let keep_running = cx
                    .update(|cx| {
                        this.update(cx, |this, cx| {
                            // Flush any pending navigation delta
                            if let Some((dir, delta)) = this.nav_coalescer.flush_pending() {
                                this.apply_nav_delta(dir, delta, cx);
                            }
                            // Check if we should keep running
                            let now = std::time::Instant::now();
                            let recently_active = now.duration_since(this.nav_coalescer.last_event)
                                < NavCoalescer::WINDOW;
                            if !recently_active && this.nav_coalescer.pending_delta == 0 {
                                // No recent activity and no pending delta - stop the task
                                this.nav_coalescer.flush_task_running = false;
                                this.nav_coalescer.reset();
                                false
                            } else {
                                true
                            }
                        })
                    })
                    .unwrap_or(Ok(false))
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        })
        .detach();
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 4 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~22.2K (22,224 exact)
  Total Chars: 125,921 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     10.4K - src/app_render.rs
      6.0K - src/app_layout.rs
      3.1K - src/app_actions.rs
      2.7K - src/app_navigation.rs

---

# Expert Review Request

## Context

This is the **UI layer** for Script Kit GPUI - rendering, actions, navigation, and layout. These files handle how the app looks and responds to user input.

## Files Included

- `app_render.rs` - Main render logic, view routing, component composition
- `app_actions.rs` - User action handlers (submit, cancel, navigation)
- `app_navigation.rs` - Keyboard navigation, focus management
- `app_layout.rs` - Layout calculations for debug overlay

## What We Need Reviewed

### 1. Render Method Structure
The main render routes to different views:
```rust
match &self.current_view {
    AppView::ScriptList => self.render_script_list(cx),
    AppView::ArgPrompt { .. } => self.render_arg_prompt(cx),
    // ... 16 view types
}
```

**Questions:**
- Should views be separate GPUI entities instead of methods?
- Is conditional rendering efficient?
- How can we reduce render method size?

### 2. Action Handlers
Actions like submit, cancel, escape:
```rust
fn handle_submit(&mut self, cx: &mut Context<Self>) {
    match &self.current_view {
        AppView::ArgPrompt { .. } => self.submit_arg(cx),
        // ...
    }
}
```

**Questions:**
- Should actions be decoupled from views?
- Is there a command pattern we should use?
- How do we handle action conflicts?

### 3. Keyboard Navigation
Arrow keys, tab, enter handling:
```rust
fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
    match event.keystroke.key.as_str() {
        "up" | "arrowup" => self.move_selection_up(cx),
        // ...
    }
}
```

**Questions:**
- Is our key handling complete for all layouts?
- Should we support vim-style navigation?
- How do we handle key repeat?

### 4. Focus Management
We track and transfer focus:
```rust
self.focus_handle.focus(window);
```

**Questions:**
- Is focus correctly transferred on view changes?
- How do we handle focus for nested components?
- Should we have a focus stack?

### 5. Layout Calculations
For debug overlay, we calculate component bounds:
```rust
fn build_component_bounds(&self, window_size: Size<Pixels>) -> Vec<ComponentBounds>
```

**Questions:**
- Is this manual calculation necessary?
- Can GPUI provide this information?
- How accurate are the bounds?

## Specific Code Areas of Concern

1. **View-specific render methods** - Duplication across views
2. **Event coalescing** - 20ms window for rapid keys
3. **Scroll handling** - `UniformListScrollHandle` usage
4. **Conditional styling** - `.when()` chains

## GPUI Patterns

We want to ensure we're using GPUI idiomatically:

**Questions:**
- Should we use more `Entity<T>` for components?
- Are we over-using `cx.listener()`?
- Is our element composition efficient?

## Deliverables Requested

1. **Render architecture review** - Methods vs. entities
2. **Action pattern recommendation** - Better dispatch approach
3. **Keyboard handling audit** - Completeness and correctness
4. **Performance analysis** - Render overhead
5. **Code organization** - How to split these large files

Thank you for your expertise!
