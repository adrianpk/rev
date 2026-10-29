/// Implemented native Org outline operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum OrgCommand {
    Cycle,
    GlobalCycle,
    Overview,
    Contents,
    ShowAll,
    HideSubtree,
    ShowChildren,
    ShowSubtree,
    HideDrawer,
    ShowDrawer,
    ToggleDrawer,
    InsertHeading,
    NextHeading,
    PreviousHeading,
    ForwardHeading,
    BackwardHeading,
    ParentHeading,
    MetaReturn,
    PromoteSubtree,
    DemoteSubtree,
    MoveSubtreeUp,
    MoveSubtreeDown,
    MetaLeft,
    MetaRight,
    MetaUp,
    MetaDown,
    InsertItem,
    IndentItem,
    OutdentItem,
    IndentItemTree,
    OutdentItemTree,
    MoveItemUp,
    MoveItemDown,
    RenumberList,
    CtrlCCtrlC,
    ToggleCheckbox,
    ToggleRadioButton,
    ToggleCheckboxPresence,
    UpdateStatisticsCookies,
    InsertCheckboxItem,
    SetProperty,
    DeleteProperty,
    InspectProperty,
    EditPropertyOwner,
    SetTags,
    RemoveTag,
    InspectTags,
    MatchTags,
    Todo,
    NextTodo,
    PreviousTodo,
    SetTodo,
    InspectTodo,
    Priority,
    PriorityUp,
    PriorityDown,
    InspectPriority,
    SetEffort,
    IncEffort,
    InspectEffort,
    Timestamp,
    TimestampInactive,
    Schedule,
    Deadline,
    GotoCalendar,
    DateFromCalendar,
    CalendarForwardDay,
    CalendarBackwardDay,
    CalendarForwardWeek,
    CalendarBackwardWeek,
    CalendarForwardMonth,
    CalendarBackwardMonth,
    CalendarForwardYear,
    CalendarBackwardYear,
    CalendarScrollThreeMonthsLeft,
    CalendarScrollThreeMonthsRight,
    CalendarGotoToday,
    CalendarSelect,
    TimestampUp,
    TimestampDown,
    TimestampUpDay,
    TimestampDownDay,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    ClockIn,
    ClockOut,
    ClockCancel,
    ClockGoto,
    ResolveClocks,
    ClockInLast,
    ClockDisplay,
    ClockReport,
    UpdateDynamicBlock,
    TimerStart,
    TimerSet,
    TimerInsert,
    TimerItem,
    TimerPause,
    TimerStop,
    TimerShow,
    HabitGraph,
    HabitToggle,
    HabitTodayOnly,
    HabitAllToday,
    HabitDate,
    HabitWindow,
    Agenda,
    AgendaDay,
    AgendaWeek,
    AgendaNext,
    AgendaPrevious,
    AgendaToday,
    AgendaRefresh,
    AgendaVisit,
    AgendaDate,
    TodoList,
    SearchView,
    TagsView,
    AgendaCustom,
    AgendaFilter,
    AgendaFilterReset,
    AgendaSort,
    AgendaColumns,
    AgendaMonth,
    AgendaYear,
    AgendaSpan,
    AgendaMark,
    AgendaUnmark,
    AgendaMarkAll,
    AgendaUnmarkAll,
    AgendaBulk,
    AgendaResults,
    AgendaUndo,
    Capture,
    CaptureKill,
    CaptureFinalize,
    Refile,
    RefileCopy,
    RefileResults,
    ArchiveSubtree,
    ToggleArchiveTag,
    ArchiveSibling,
    ArchiveResults,
    StoreLink,
    InsertLink,
    OpenLink,
    LinkBack,
    LinkMark,
    NextLink,
    PreviousLink,
    Attach,
    AttachFile,
    AttachCopy,
    AttachMove,
    AttachLink,
    AttachOpen,
    AttachOpenNative,
    AttachList,
    AttachDirectory,
    AttachResults,
    FootnoteAction,
    FootnoteNew,
    FootnoteRenumber,
    Emphasize,
    InsertEntity,
    EditAffiliated,
    TableCreate,
    TableAlign,
    TableNextField,
    TablePreviousField,
    TableNextRow,
    TableBeginningField,
    TableEndField,
    TableInsertRow,
    TableInsertHline,
    Return,
    TableKillRow,
    TableMoveRowUp,
    TableMoveRowDown,
    TableInsertColumn,
    TableDeleteColumn,
    TableMoveColumnLeft,
    TableMoveColumnRight,
    TableSort,
    ListToTable,
    TableToList,
    TableRecalculate,
    TableRecalculateCell,
    TableRecalculateColumn,
    TableImport,
    TableExport,
    Columns,
    ColumnsInsertDblock,
    ExportText,
    ExportUtf8,
    ExportMarkdown,
    ExportOrg,
}
impl OrgCommand {
    pub const ALL: [Self; 199] = [
        Self::Cycle,
        Self::GlobalCycle,
        Self::Overview,
        Self::Contents,
        Self::ShowAll,
        Self::HideSubtree,
        Self::ShowChildren,
        Self::ShowSubtree,
        Self::HideDrawer,
        Self::ShowDrawer,
        Self::ToggleDrawer,
        Self::InsertHeading,
        Self::NextHeading,
        Self::PreviousHeading,
        Self::ForwardHeading,
        Self::BackwardHeading,
        Self::ParentHeading,
        Self::MetaReturn,
        Self::PromoteSubtree,
        Self::DemoteSubtree,
        Self::MoveSubtreeUp,
        Self::MoveSubtreeDown,
        Self::MetaLeft,
        Self::MetaRight,
        Self::MetaUp,
        Self::MetaDown,
        Self::InsertItem,
        Self::IndentItem,
        Self::OutdentItem,
        Self::IndentItemTree,
        Self::OutdentItemTree,
        Self::MoveItemUp,
        Self::MoveItemDown,
        Self::RenumberList,
        Self::CtrlCCtrlC,
        Self::ToggleCheckbox,
        Self::ToggleRadioButton,
        Self::ToggleCheckboxPresence,
        Self::UpdateStatisticsCookies,
        Self::InsertCheckboxItem,
        Self::SetProperty,
        Self::DeleteProperty,
        Self::InspectProperty,
        Self::EditPropertyOwner,
        Self::SetTags,
        Self::RemoveTag,
        Self::InspectTags,
        Self::MatchTags,
        Self::Todo,
        Self::NextTodo,
        Self::PreviousTodo,
        Self::SetTodo,
        Self::InspectTodo,
        Self::Priority,
        Self::PriorityUp,
        Self::PriorityDown,
        Self::InspectPriority,
        Self::SetEffort,
        Self::IncEffort,
        Self::InspectEffort,
        Self::Timestamp,
        Self::TimestampInactive,
        Self::Schedule,
        Self::Deadline,
        Self::GotoCalendar,
        Self::DateFromCalendar,
        Self::CalendarForwardDay,
        Self::CalendarBackwardDay,
        Self::CalendarForwardWeek,
        Self::CalendarBackwardWeek,
        Self::CalendarForwardMonth,
        Self::CalendarBackwardMonth,
        Self::CalendarForwardYear,
        Self::CalendarBackwardYear,
        Self::CalendarScrollThreeMonthsLeft,
        Self::CalendarScrollThreeMonthsRight,
        Self::CalendarGotoToday,
        Self::CalendarSelect,
        Self::TimestampUp,
        Self::TimestampDown,
        Self::TimestampUpDay,
        Self::TimestampDownDay,
        Self::ShiftUp,
        Self::ShiftDown,
        Self::ShiftLeft,
        Self::ShiftRight,
        Self::ClockIn,
        Self::ClockOut,
        Self::ClockCancel,
        Self::ClockGoto,
        Self::ResolveClocks,
        Self::ClockInLast,
        Self::ClockDisplay,
        Self::ClockReport,
        Self::UpdateDynamicBlock,
        Self::TimerStart,
        Self::TimerSet,
        Self::TimerInsert,
        Self::TimerItem,
        Self::TimerPause,
        Self::TimerStop,
        Self::TimerShow,
        Self::HabitGraph,
        Self::HabitToggle,
        Self::HabitTodayOnly,
        Self::HabitAllToday,
        Self::HabitDate,
        Self::HabitWindow,
        Self::Agenda,
        Self::AgendaDay,
        Self::AgendaWeek,
        Self::AgendaNext,
        Self::AgendaPrevious,
        Self::AgendaToday,
        Self::AgendaRefresh,
        Self::AgendaVisit,
        Self::AgendaDate,
        Self::TodoList,
        Self::SearchView,
        Self::TagsView,
        Self::AgendaCustom,
        Self::AgendaFilter,
        Self::AgendaFilterReset,
        Self::AgendaSort,
        Self::AgendaColumns,
        Self::AgendaMonth,
        Self::AgendaYear,
        Self::AgendaSpan,
        Self::AgendaMark,
        Self::AgendaUnmark,
        Self::AgendaMarkAll,
        Self::AgendaUnmarkAll,
        Self::AgendaBulk,
        Self::AgendaResults,
        Self::AgendaUndo,
        Self::Capture,
        Self::CaptureKill,
        Self::CaptureFinalize,
        Self::Refile,
        Self::RefileCopy,
        Self::RefileResults,
        Self::ArchiveSubtree,
        Self::ToggleArchiveTag,
        Self::ArchiveSibling,
        Self::ArchiveResults,
        Self::StoreLink,
        Self::InsertLink,
        Self::OpenLink,
        Self::LinkBack,
        Self::LinkMark,
        Self::NextLink,
        Self::PreviousLink,
        Self::Attach,
        Self::AttachFile,
        Self::AttachCopy,
        Self::AttachMove,
        Self::AttachLink,
        Self::AttachOpen,
        Self::AttachOpenNative,
        Self::AttachList,
        Self::AttachDirectory,
        Self::AttachResults,
        Self::FootnoteAction,
        Self::FootnoteNew,
        Self::FootnoteRenumber,
        Self::Emphasize,
        Self::InsertEntity,
        Self::EditAffiliated,
        Self::TableCreate,
        Self::TableAlign,
        Self::TableNextField,
        Self::TablePreviousField,
        Self::TableNextRow,
        Self::TableBeginningField,
        Self::TableEndField,
        Self::TableInsertRow,
        Self::TableInsertHline,
        Self::Return,
        Self::TableKillRow,
        Self::TableMoveRowUp,
        Self::TableMoveRowDown,
        Self::TableInsertColumn,
        Self::TableDeleteColumn,
        Self::TableMoveColumnLeft,
        Self::TableMoveColumnRight,
        Self::TableSort,
        Self::ListToTable,
        Self::TableToList,
        Self::TableRecalculate,
        Self::TableRecalculateCell,
        Self::TableRecalculateColumn,
        Self::TableImport,
        Self::TableExport,
        Self::Columns,
        Self::ColumnsInsertDblock,
        Self::ExportText,
        Self::ExportUtf8,
        Self::ExportMarkdown,
        Self::ExportOrg,
    ];
    pub fn name(self) -> &'static str {
        [
            "org-cycle",
            "org-shifttab",
            "org-overview",
            "org-content",
            "org-show-all",
            "org-hide-subtree",
            "org-show-children",
            "org-show-subtree",
            "org-hide-drawer",
            "org-show-drawer",
            "org-toggle-drawer",
            "org-insert-heading",
            "org-next-visible-heading",
            "org-previous-visible-heading",
            "org-forward-heading-same-level",
            "org-backward-heading-same-level",
            "org-up-heading-safe",
            "org-meta-return",
            "org-promote-subtree",
            "org-demote-subtree",
            "org-move-subtree-up",
            "org-move-subtree-down",
            "org-metaleft",
            "org-metaright",
            "org-metaup",
            "org-metadown",
            "org-insert-item",
            "org-indent-item",
            "org-outdent-item",
            "org-indent-item-tree",
            "org-outdent-item-tree",
            "org-move-item-up",
            "org-move-item-down",
            "org-renumber-list",
            "org-ctrl-c-ctrl-c",
            "org-toggle-checkbox",
            "org-toggle-radio-button",
            "org-toggle-checkbox-presence",
            "org-update-statistics-cookies",
            "org-insert-item-checkbox",
            "org-set-property",
            "org-delete-property",
            "org-inspect-property",
            "org-edit-property-owner",
            "org-set-tags-command",
            "org-remove-tag",
            "org-inspect-tags",
            "org-match-tags",
            "org-todo",
            "org-todo-next",
            "org-todo-previous",
            "org-todo-set",
            "org-inspect-todo",
            "org-priority",
            "org-priority-up",
            "org-priority-down",
            "org-inspect-priority",
            "org-set-effort",
            "org-inc-effort",
            "org-inspect-effort",
            "org-timestamp",
            "org-timestamp-inactive",
            "org-schedule",
            "org-deadline",
            "org-goto-calendar",
            "org-date-from-calendar",
            "org-calendar-forward-day",
            "org-calendar-backward-day",
            "org-calendar-forward-week",
            "org-calendar-backward-week",
            "org-calendar-forward-month",
            "org-calendar-backward-month",
            "org-calendar-forward-year",
            "org-calendar-backward-year",
            "org-calendar-scroll-three-months-left",
            "org-calendar-scroll-three-months-right",
            "org-calendar-goto-today",
            "org-calendar-select",
            "org-timestamp-up",
            "org-timestamp-down",
            "org-timestamp-up-day",
            "org-timestamp-down-day",
            "org-shiftup",
            "org-shiftdown",
            "org-shiftleft",
            "org-shiftright",
            "org-clock-in",
            "org-clock-out",
            "org-clock-cancel",
            "org-clock-goto",
            "org-resolve-clocks",
            "org-clock-in-last",
            "org-clock-display",
            "org-clock-report",
            "org-dblock-update",
            "org-timer-start",
            "org-timer-set-timer",
            "org-timer",
            "org-timer-item",
            "org-timer-pause-or-continue",
            "org-timer-stop",
            "org-timer-show-remaining-time",
            "org-habit-show-graph",
            "org-habit-toggle-habits",
            "org-habit-toggle-habits-only-for-today",
            "org-habit-toggle-show-all-today",
            "org-habit-graph-date",
            "org-habit-set-graph-window",
            "org-agenda",
            "org-agenda-day-view",
            "org-agenda-week-view",
            "org-agenda-later",
            "org-agenda-earlier",
            "org-agenda-goto-today",
            "org-agenda-redo",
            "org-agenda-switch-to",
            "org-agenda-goto-date",
            "org-todo-list",
            "org-search-view",
            "org-tags-view",
            "org-agenda-custom-view",
            "org-agenda-filter",
            "org-agenda-filter-remove-all",
            "org-agenda-sort",
            "org-agenda-columns",
            "org-agenda-month-view",
            "org-agenda-year-view",
            "org-agenda-span",
            "org-agenda-bulk-mark",
            "org-agenda-bulk-unmark",
            "org-agenda-bulk-mark-all",
            "org-agenda-bulk-unmark-all",
            "org-agenda-bulk-action",
            "org-agenda-action-results",
            "org-agenda-undo",
            "org-capture",
            "org-capture-kill",
            "org-capture-finalize",
            "org-refile",
            "org-refile-copy",
            "org-refile-results",
            "org-archive-subtree",
            "org-toggle-archive-tag",
            "org-archive-to-archive-sibling",
            "org-archive-results",
            "org-store-link",
            "org-insert-link",
            "org-open-at-point",
            "org-mark-ring-goto",
            "org-mark-ring-push",
            "org-next-link",
            "org-previous-link",
            "org-attach",
            "org-attach-attach",
            "org-attach-copy",
            "org-attach-move",
            "org-attach-link",
            "org-attach-open",
            "org-attach-open-in-emacs",
            "org-attach-reveal-in-emacs",
            "org-attach-set-directory",
            "org-attach-results",
            "org-footnote-action",
            "org-footnote-new",
            "org-footnote-renumber",
            "org-emphasize",
            "org-insert-entity",
            "org-edit-affiliated-keyword",
            "org-table-create",
            "org-table-align",
            "org-table-next-field",
            "org-table-previous-field",
            "org-table-next-row",
            "org-table-beginning-of-field",
            "org-table-end-of-field",
            "org-table-insert-row",
            "org-table-insert-hline",
            "org-return",
            "org-table-kill-row",
            "org-table-move-row-up",
            "org-table-move-row-down",
            "org-table-insert-column",
            "org-table-delete-column",
            "org-table-move-column-left",
            "org-table-move-column-right",
            "org-table-sort-lines",
            "org-list-to-table",
            "org-table-to-list",
            "org-table-recalculate",
            "org-table-recalculate-cell",
            "org-table-recalculate-column",
            "org-table-import",
            "org-table-export",
            "org-columns",
            "org-columns-insert-dblock",
            "org-export-to-text",
            "org-export-to-unicode",
            "org-export-to-markdown",
            "org-export-to-org",
        ][self as usize]
    }

    pub(crate) fn effort(self) -> bool {
        matches!(
            self,
            Self::SetEffort | Self::IncEffort | Self::InspectEffort
        )
    }
    pub(crate) fn estimate(self) -> bool {
        self.effort()
            || matches!(
                self,
                Self::Priority | Self::PriorityUp | Self::PriorityDown | Self::InspectPriority
            )
    }
    pub(crate) fn task(self) -> bool {
        matches!(
            self,
            Self::Todo | Self::NextTodo | Self::PreviousTodo | Self::SetTodo | Self::InspectTodo
        )
    }
    pub(crate) fn tags(self) -> bool {
        matches!(
            self,
            Self::SetTags | Self::RemoveTag | Self::InspectTags | Self::MatchTags
        )
    }
    pub(crate) fn property(self) -> bool {
        matches!(
            self,
            Self::SetProperty
                | Self::DeleteProperty
                | Self::InspectProperty
                | Self::EditPropertyOwner
        )
    }

    pub(crate) fn list_insertion(self) -> bool {
        matches!(
            self,
            Self::MetaReturn | Self::InsertItem | Self::InsertCheckboxItem
        )
    }
    pub(crate) fn list_indentation(self) -> bool {
        matches!(
            self,
            Self::IndentItem
                | Self::OutdentItem
                | Self::IndentItemTree
                | Self::OutdentItemTree
                | Self::MetaLeft
                | Self::MetaRight
        )
    }
    pub(crate) fn list_movement(self) -> bool {
        matches!(
            self,
            Self::MoveItemUp | Self::MoveItemDown | Self::MetaUp | Self::MetaDown
        )
    }
    pub(crate) fn checkbox(self) -> bool {
        matches!(
            self,
            Self::CtrlCCtrlC
                | Self::ToggleCheckbox
                | Self::ToggleRadioButton
                | Self::ToggleCheckboxPresence
                | Self::UpdateStatisticsCookies
                | Self::InsertCheckboxItem
        )
    }
    pub(crate) fn list_only(self) -> bool {
        self.checkbox()
            || matches!(
                self,
                Self::InsertItem
                    | Self::IndentItem
                    | Self::OutdentItem
                    | Self::IndentItemTree
                    | Self::OutdentItemTree
                    | Self::MoveItemUp
                    | Self::MoveItemDown
                    | Self::RenumberList
            )
    }
    pub(crate) fn contextual_list(self) -> bool {
        matches!(
            self,
            Self::MetaReturn | Self::MetaLeft | Self::MetaRight | Self::MetaUp | Self::MetaDown
        )
    }
    pub(crate) fn subtree(self) -> Option<Self> {
        match self {
            Self::MetaLeft => Some(Self::PromoteSubtree),
            Self::MetaRight => Some(Self::DemoteSubtree),
            Self::MetaUp => Some(Self::MoveSubtreeUp),
            Self::MetaDown => Some(Self::MoveSubtreeDown),
            command if command.edits_subtree() => Some(command),
            _ => None,
        }
    }
    pub(crate) fn edits_subtree(self) -> bool {
        matches!(
            self,
            Self::PromoteSubtree
                | Self::DemoteSubtree
                | Self::MoveSubtreeUp
                | Self::MoveSubtreeDown
        )
    }
}

impl OrgCommand {
    pub(crate) fn timestamp(self) -> bool {
        (Self::Timestamp as usize..=Self::DateFromCalendar as usize).contains(&(self as usize))
            || (Self::TimestampUp as usize..=Self::TimestampDownDay as usize)
                .contains(&(self as usize))
    }
    pub(crate) fn calendar(self) -> bool {
        (Self::CalendarForwardDay as usize..=Self::CalendarSelect as usize)
            .contains(&(self as usize))
    }
}

impl OrgCommand {
    pub(crate) fn timestamp_step(self) -> bool {
        (Self::TimestampUp as usize..=Self::ShiftRight as usize).contains(&(self as usize))
    }
    pub(crate) fn timestamp_context(self) -> bool {
        matches!(
            self,
            Self::ShiftUp | Self::ShiftDown | Self::ShiftLeft | Self::ShiftRight | Self::CtrlCCtrlC
        )
    }
}

impl OrgCommand {
    pub(crate) fn clock(self) -> bool {
        matches!(
            self,
            Self::ClockIn
                | Self::ClockOut
                | Self::ClockCancel
                | Self::ClockGoto
                | Self::ResolveClocks
                | Self::ClockInLast
        )
    }
}

impl OrgCommand {
    pub(crate) fn clock_source(self) -> bool {
        matches!(
            self,
            Self::ClockDisplay
                | Self::ClockReport
                | Self::UpdateDynamicBlock
                | Self::ClockIn
                | Self::ResolveClocks
        )
    }
}

impl OrgCommand {
    pub(crate) fn timer_control(self) -> bool {
        matches!(
            self,
            Self::TimerStart
                | Self::TimerSet
                | Self::TimerPause
                | Self::TimerStop
                | Self::TimerShow
        )
    }
    pub(crate) fn timer_insertion(self) -> bool {
        matches!(self, Self::TimerInsert | Self::TimerItem)
    }
}

impl OrgCommand {
    pub(crate) fn session_clock(self) -> bool {
        self.clock() && !matches!(self, Self::ClockIn | Self::ResolveClocks)
    }
}

impl OrgCommand {
    pub(crate) fn habit(self) -> bool {
        matches!(
            self,
            Self::HabitGraph
                | Self::HabitToggle
                | Self::HabitTodayOnly
                | Self::HabitAllToday
                | Self::HabitDate
                | Self::HabitWindow
        )
    }
}

impl OrgCommand {
    pub(crate) fn agenda_bulk(self) -> bool {
        (Self::AgendaMark as usize..=Self::AgendaUndo as usize).contains(&(self as usize))
    }
    pub(crate) fn agenda_source_action(self) -> bool {
        self.task()
            || self.estimate()
            || matches!(
                self,
                Self::SetTags
                    | Self::RemoveTag
                    | Self::Schedule
                    | Self::Deadline
                    | Self::ClockIn
                    | Self::ClockOut
                    | Self::ClockCancel
            )
    }
    pub(crate) fn agenda_custom(self) -> bool {
        (Self::AgendaCustom as usize..=Self::AgendaSpan as usize).contains(&(self as usize))
    }
    pub(crate) fn agenda_query(self) -> bool {
        matches!(self, Self::TodoList | Self::SearchView | Self::TagsView)
    }
    pub(crate) fn agenda_control(self) -> bool {
        (Self::AgendaDay as usize..=Self::AgendaDate as usize).contains(&(self as usize))
    }
}

impl OrgCommand {
    pub(crate) fn archive(self) -> bool {
        matches!(
            self,
            Self::ArchiveSubtree | Self::ToggleArchiveTag | Self::ArchiveSibling
        )
    }
}

impl OrgCommand {
    pub(crate) fn links(self) -> bool {
        matches!(
            self,
            Self::StoreLink
                | Self::InsertLink
                | Self::OpenLink
                | Self::NextLink
                | Self::PreviousLink
        )
    }
}

impl OrgCommand {
    pub(crate) fn attachment(self) -> bool {
        (Self::Attach as usize..=Self::AttachDirectory as usize).contains(&(self as usize))
    }
}

impl OrgCommand {
    pub(crate) fn document(self) -> bool {
        matches!(
            self,
            Self::FootnoteAction
                | Self::FootnoteNew
                | Self::FootnoteRenumber
                | Self::Emphasize
                | Self::InsertEntity
                | Self::EditAffiliated
        )
    }
}

impl OrgCommand {
    pub(crate) fn table(self) -> bool {
        matches!(
            self,
            Self::TableCreate
                | Self::TableAlign
                | Self::TableNextField
                | Self::TablePreviousField
                | Self::TableNextRow
                | Self::TableBeginningField
                | Self::TableEndField
                | Self::TableInsertRow
                | Self::TableInsertHline
                | Self::Return
                | Self::TableKillRow
                | Self::TableMoveRowUp
                | Self::TableMoveRowDown
                | Self::TableInsertColumn
                | Self::TableDeleteColumn
                | Self::TableMoveColumnLeft
                | Self::TableMoveColumnRight
                | Self::TableSort
                | Self::ListToTable
                | Self::TableToList
                | Self::TableRecalculate
                | Self::TableRecalculateCell
                | Self::TableRecalculateColumn
                | Self::TableImport
                | Self::TableExport
        )
    }
    pub(crate) fn recalculation(self) -> bool {
        matches!(
            self,
            Self::TableRecalculate | Self::TableRecalculateCell | Self::TableRecalculateColumn
        )
    }
}

impl OrgCommand {
    pub(crate) fn export(self) -> Option<crate::config::ExportBackend> {
        use crate::config::ExportBackend as B;
        match self {
            Self::ExportText => Some(B::Text),
            Self::ExportUtf8 => Some(B::Utf8),
            Self::ExportMarkdown => Some(B::Markdown),
            Self::ExportOrg => Some(B::Org),
            _ => None,
        }
    }
}
