// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Uuid,
        task_id -> Uuid,
        author_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        body -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        edited_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    cycles (id) {
        id -> Uuid,
        team_id -> Uuid,
        seq -> Int4,
        name -> Text,
        starts_on -> Date,
        ends_on -> Date,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_authorizations (id) {
        id -> Uuid,
        device_code_hash -> Bytea,
        user_code -> Text,
        user_id -> Nullable<Uuid>,
        status -> Text,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        last_polled_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    goals (id) {
        id -> Uuid,
        user_id -> Uuid,
        team_id -> Nullable<Uuid>,
        project_id -> Nullable<Uuid>,
        name -> Text,
        description -> Text,
        status -> Text,
        target_date -> Nullable<Date>,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    habit_checkins (habit_id, day) {
        habit_id -> Uuid,
        day -> Date,
        times -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    habits (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        description -> Text,
        weekdays -> Nullable<Array<Nullable<Int2>>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        archived_at -> Nullable<Timestamptz>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        unit -> Nullable<Text>,
        tracking -> Text,
        position -> Float8,
        goal_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    horizons (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        name -> Text,
        valid_days -> Nullable<Int4>,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    labels (id) {
        id -> Uuid,
        team_id -> Nullable<Uuid>,
        name -> Text,
        color -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    milestone_members (milestone_id, user_id) {
        milestone_id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    milestones (id) {
        id -> Uuid,
        project_id -> Uuid,
        name -> Text,
        description -> Text,
        target_date -> Nullable<Date>,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    oauth_states (state) {
        state -> Text,
        pkce_verifier -> Text,
        redirect_uri -> Text,
        device_code_hash -> Nullable<Bytea>,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    plan_levels (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        period -> Text,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    plan_requirement_habits (requirement_id, habit_id) {
        requirement_id -> Uuid,
        habit_id -> Uuid,
    }
}

diesel::table! {
    plan_requirements (id) {
        id -> Uuid,
        level_id -> Uuid,
        name -> Nullable<Text>,
        quota -> Float8,
        measure -> Text,
        position -> Float8,
    }
}

diesel::table! {
    plan_tasks (level_id, task_id, period_start) {
        level_id -> Uuid,
        task_id -> Uuid,
        period_start -> Date,
    }
}

diesel::table! {
    project_members (project_id, user_id) {
        project_id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    projects (id) {
        id -> Uuid,
        team_id -> Nullable<Uuid>,
        creator_id -> Uuid,
        name -> Text,
        description -> Text,
        status -> Text,
        target_date -> Nullable<Date>,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        goal_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Bytea,
        prev_hash -> Nullable<Bytea>,
        client -> Text,
        user_agent -> Nullable<Text>,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        last_used_at -> Timestamptz,
    }
}

diesel::table! {
    task_goals (task_id, goal_id) {
        task_id -> Uuid,
        goal_id -> Uuid,
    }
}

diesel::table! {
    task_labels (task_id, label_id) {
        task_id -> Uuid,
        label_id -> Uuid,
    }
}

diesel::table! {
    task_relations (from_task_id, to_task_id, kind) {
        from_task_id -> Uuid,
        to_task_id -> Uuid,
        kind -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    tasks (id) {
        id -> Uuid,
        team_id -> Nullable<Uuid>,
        seq -> Nullable<Int4>,
        creator_id -> Uuid,
        assignee_id -> Nullable<Uuid>,
        title -> Text,
        description -> Text,
        state_id -> Uuid,
        priority -> Int2,
        estimate -> Nullable<Float8>,
        parent_id -> Nullable<Uuid>,
        project_id -> Nullable<Uuid>,
        milestone_id -> Nullable<Uuid>,
        cycle_id -> Nullable<Uuid>,
        is_milestone_goal -> Bool,
        horizon_id -> Nullable<Uuid>,
        horizon_set_at -> Nullable<Timestamptz>,
        due_date -> Nullable<Date>,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        canceled_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    team_members (team_id, user_id) {
        team_id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    teams (id) {
        id -> Uuid,
        key -> Text,
        name -> Text,
        next_seq -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        google_sub -> Text,
        email -> Text,
        display_name -> Text,
        avatar_url -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workflow_states (id) {
        id -> Uuid,
        team_id -> Nullable<Uuid>,
        name -> Text,
        category -> Text,
        color -> Text,
        position -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(comments -> tasks (task_id));
diesel::joinable!(comments -> users (author_id));
diesel::joinable!(cycles -> teams (team_id));
diesel::joinable!(device_authorizations -> users (user_id));
diesel::joinable!(goals -> teams (team_id));
diesel::joinable!(goals -> users (user_id));
diesel::joinable!(habit_checkins -> habits (habit_id));
diesel::joinable!(habits -> goals (goal_id));
diesel::joinable!(habits -> users (user_id));
diesel::joinable!(horizons -> users (user_id));
diesel::joinable!(labels -> teams (team_id));
diesel::joinable!(milestone_members -> milestones (milestone_id));
diesel::joinable!(milestone_members -> users (user_id));
diesel::joinable!(milestones -> projects (project_id));
diesel::joinable!(plan_levels -> users (user_id));
diesel::joinable!(plan_requirement_habits -> habits (habit_id));
diesel::joinable!(plan_requirement_habits -> plan_requirements (requirement_id));
diesel::joinable!(plan_requirements -> plan_levels (level_id));
diesel::joinable!(plan_tasks -> plan_levels (level_id));
diesel::joinable!(plan_tasks -> tasks (task_id));
diesel::joinable!(project_members -> projects (project_id));
diesel::joinable!(project_members -> users (user_id));
diesel::joinable!(projects -> teams (team_id));
diesel::joinable!(projects -> users (creator_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(task_goals -> goals (goal_id));
diesel::joinable!(task_goals -> tasks (task_id));
diesel::joinable!(task_labels -> labels (label_id));
diesel::joinable!(task_labels -> tasks (task_id));
diesel::joinable!(tasks -> cycles (cycle_id));
diesel::joinable!(tasks -> horizons (horizon_id));
diesel::joinable!(tasks -> milestones (milestone_id));
diesel::joinable!(tasks -> projects (project_id));
diesel::joinable!(tasks -> teams (team_id));
diesel::joinable!(tasks -> workflow_states (state_id));
diesel::joinable!(team_members -> teams (team_id));
diesel::joinable!(team_members -> users (user_id));
diesel::joinable!(workflow_states -> teams (team_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    cycles,
    device_authorizations,
    goals,
    habit_checkins,
    habits,
    horizons,
    labels,
    milestone_members,
    milestones,
    oauth_states,
    plan_levels,
    plan_requirement_habits,
    plan_requirements,
    plan_tasks,
    project_members,
    projects,
    sessions,
    task_goals,
    task_labels,
    task_relations,
    tasks,
    team_members,
    teams,
    users,
    workflow_states,
);
