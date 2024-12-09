use editor::Editor;
use gpui::{
    actions, div, prelude::*, AnyElement, AppContext, DismissEvent, EventEmitter, FocusHandle,
    FocusableView, Model, Render, Subscription, View, ViewContext, WeakView,
};
use project::Project;
use std::path::PathBuf;
use ui::{prelude::*, ListItem};
use workspace::{ModalLayer, ModalView, Workspace};

actions!(go_to_file, [Toggle]);

pub struct GoToFile {
    number_editor: View<Editor>,
    project: Model<Project>,
    current_text: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl ModalView for GoToFile {}

impl FocusableView for GoToFile {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.number_editor.focus_handle(cx)
    }
}

impl EventEmitter<DismissEvent> for GoToFile {}

impl GoToFile {
    pub fn register(workspace: &mut Workspace, cx: &mut ViewContext<Workspace>) {
        workspace.register_action(move |workspace, _: &Toggle, cx| {
            Self::toggle(workspace, cx);
        });
    }

    fn toggle(workspace: &mut Workspace, cx: &mut ViewContext<Workspace>) -> Option<()> {
        let project = workspace.project().clone();
        workspace.toggle_modal(cx, |cx| Self::new(project, cx));
        Some(())
    }

    pub fn new(project: Model<Project>, cx: &mut ViewContext<Self>) -> Self {
        let number_editor = cx.new_view(|cx| {
            let mut editor = Editor::single_line(cx);
            editor.set_placeholder_text("Enter file number...", cx);
            editor
        });

        let current_text = format!("Total files: {}", 10).into();
        let number_editor_change = cx.subscribe(&number_editor, Self::on_number_editor_event);

        Self {
            number_editor,
            project,
            current_text,
            _subscriptions: vec![number_editor_change],
        }
    }

    fn on_number_editor_event(
        &mut self,
        _: View<Editor>,
        event: &editor::EditorEvent,
        cx: &mut ViewContext<Self>,
    ) {
        match event {
            editor::EditorEvent::Blurred => cx.emit(DismissEvent),
            editor::EditorEvent::BufferEdited { .. } => self.update_current_text(cx),
            _ => {}
        }
    }

    fn update_current_text(&mut self, cx: &mut ViewContext<Self>) {
        let input = self.number_editor.read(cx).text(cx);
        if let Ok(number) = input.trim().parse::<usize>() {}
        self.current_text = format!("Enter a number between 1 and {}", 10).into();
        cx.notify();
    }

    fn cancel(&mut self, _: &menu::Cancel, cx: &mut ViewContext<Self>) {
        cx.emit(DismissEvent);
    }

    fn confirm(&mut self, _: &menu::Confirm, cx: &mut ViewContext<Self>) {
        let input = self.number_editor.read(cx).text(cx);
        if let Ok(file_number) = input.trim().parse::<usize>() {
            // First emit to project
            self.project.update(cx, |_, cx| {
                dbg!("Emitting open numbered file");
                cx.emit(project::Event::OpenNumberedFile { file_number })
            });
        }
        // Then dismiss the modal
        cx.emit(DismissEvent);
    }
}

impl Render for GoToFile {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .w(rems(24.))
            .elevation_2(cx)
            .key_context("GoToFile")
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            .child(
                div()
                    .border_b_1()
                    .border_color(cx.theme().colors().border_variant)
                    .px_2()
                    .py_1()
                    .child(self.number_editor.clone()),
            )
            .child(
                h_flex()
                    .px_2()
                    .py_1()
                    .gap_1()
                    .child(Label::new("File:").color(Color::Muted))
                    .child(Label::new(self.current_text.clone()).color(Color::Muted)),
            )
    }
}
