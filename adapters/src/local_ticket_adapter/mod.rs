mod adapter;
mod interpreter;
mod interpreter_errors;
mod interpreter_tests;
mod interpreter_instructions;
mod interpreter_parameters;

use std::sync::{
    Arc, Mutex
};

use tickets_rs_core::{
    LocalDatabase, 
    AppConfig, 
    Bucket, 
    Tag, 
    State, 
    Filter,
    FilterType,
    Ticket};

use tickets_rs_core::{
    TicketAdapter
};

pub struct LocalTicketAdapter {
    database: Arc<Mutex<LocalDatabase>>,
    config: Arc<Mutex<AppConfig>>,
    name: String,
    display_name: String

}

impl LocalTicketAdapter {

    pub(crate) fn prepare_database(&self, create_default_data: bool) {
            let (
                bucket_tables, 
                ticket_tables,
                state_tables,
                tag_tables,
                tagticket_tables,
                filter_tables

            ) = match self.database.lock() {
            Ok(mut lock) => {

                let buckets = lock.create_table(
                    &String::from("buckets"), vec![
                        String::from("id INTEGER PRIMARY KEY AUTOINCREMENT"),
                        String::from("name TEXT NOT NULL"), 
                        String::from("last_change INTEGER")]);

                let tickets = lock.create_table(
                    &String::from("tickets"), vec![
                        String::from("id INTEGER PRIMARY KEY AUTOINCREMENT"),
                        String::from("bucket_id INTEGER NOT NULL"), 
                        String::from("title TEXT NOT NULL"), 
                        String::from("state_name TEXT NOT NULL"), 
                        String::from("description TEXT"), 
                        String::from("created_at INTEGER"), 
                        String::from("due_at INTEGER"),
                        String::from("assigned_to TEXT")]);

                let ticket_tags = lock.create_table(
                    &String::from("ticket_tags"), vec![
                        String::from("ticket_id INTEGER NOT NULL"),
                        String::from("tag_name TEXT NOT NULL")]);

                let states = lock.create_table(
                    &String::from("states"), vec![
                        String::from("name TEXT NOT NULL PRIMARY KEY"),
                        String::from("description TEXT"),
                        String::from("sorting_order INTEGER NOT NULL")]);

                let tags = lock.create_table(
                    &String::from("tags"), vec![
                        String::from("name TEXT NOT NULL PRIMARY KEY"),
                        String::from("color TEXT NOT NULL"),
                        String::from("color_text TEXT NOT NULL")]);

                let filters = lock.create_table(
                    &String::from("filters"), vec![
                        String::from("name TEXT NOT NULL PRIMARY KEY"),
                        String::from("operation TEXT NOT NULL")]);

                (buckets, tickets, states, tags, ticket_tags, filters)
            },
            Err(_) => (false, false, false, false, false, false),
        };

        if create_default_data {
            let mut bucket_default = Bucket::default()
                .with_adapter(self)
                .with_details(0, String::from("default.bucket"));

            let mut bucket_empty = Bucket::default()
                .with_adapter(self)
                .with_details(0, String::from("empty.bucket"));

            if bucket_tables {

                match self.bucket_write(&mut bucket_default)
                 .and(self.bucket_write(&mut bucket_empty)) {
                    Ok(_) => (),
                    Err(err) => println!("Wasn't able to write buckets as default data due to {err}"),
                };
            }

            let tag_bug = Tag::default()
                .with_adapter(self)
                .with_name(String::from("bug"))
                .with_hex_colors("#321820", "#b87881");

            let tag_enhancement = Tag::default()
                .with_adapter(self)
                .with_name(String::from("enhancement"))
                .with_hex_colors("#28393e", "#8fd4d5");

            let tag_documentation = Tag::default()
                .with_adapter(self)
                .with_name(String::from("documentation"))
                .with_hex_colors("#0b2337", "#309ce8");

            let tag_wontfix = Tag::default()
                .with_adapter(self)
                .with_name(String::from("wontfix"))
                .with_hex_colors("#393c41", "#d7d7d7");

            let tag_blocker = Tag::default()
                .with_adapter(self)
                .with_name(String::from("blocker"))
                .with_hex_colors("#a71a35", "#ffd9df");

            let tag_high_prio = Tag::default()
                .with_adapter(self)
                .with_name(String::from("high priority"))
                .with_hex_colors("#a7581a", "#ffecd9");

            let tag_low_prio = Tag::default()
                .with_adapter(self)
                .with_name(String::from("low priority"))
                .with_hex_colors("#4e542a", "#c9d5b6");

            if tag_tables {
                match self.tag_write(&tag_bug)
                 .and(self.tag_write(&tag_enhancement))
                 .and(self.tag_write(&tag_documentation))
                 .and(self.tag_write(&tag_wontfix))
                 .and(self.tag_write(&tag_blocker))
                 .and(self.tag_write(&tag_high_prio))
                 .and(self.tag_write(&tag_low_prio)) {
                    Ok(_) => (),
                    Err(err) => println!("Wasn't able to write tags as default data due to {err}"),
                };
            };

            let state_new = State::default()
                .with_adapter(self)
                .with_name(String::from("new"))
                .with_description(String::from(concat!("A Ticket qualifies as new, if it has been created, ",
                    "but nobody has worked on it yet.")))
                .with_order(0);

            let state_pause = State::default()
                .with_adapter(self)
                .with_name(String::from("pause"))
                .with_description(String::from(concat!("A Ticket qualifies as paused, if it has been worked",
                    " on, but the work has been paused for now.")))
                .with_order(0);

            let state_open = State::default()
                .with_adapter(self)
                .with_name(String::from("open"))
                .with_description(String::from("A Ticket qualifies as open, if it is actively worked on."))
                .with_order(1);

            let state_done = State::default()
                .with_adapter(self)
                .with_name(String::from("done"))
                .with_description(String::from("A Ticket is done, when the content of it has been finished."))
                .with_order(2);

            let state_live = State::default()
                .with_adapter(self)
                .with_name(String::from("live"))
                .with_description(String::from(concat!("Specifically on Running Systems with multiple ",
                    "Branches it is important to not, if a ticket is live. A Ticket qualifies ",
                    "as live, if it's feature is used in the corresponding production environment.")))
                .with_order(3);

            if state_tables {

                match self.state_write(&state_new)
                 .and(self.state_write(&state_pause))
                 .and(self.state_write(&state_open))
                 .and(self.state_write(&state_done))
                 .and(self.state_write(&state_live)) {
                    Ok(_) => (),
                    Err(err) => println!("Wasn't able to write states as default data due to {err}"),
                }
            }

            if filter_tables {

                let filter_state_new = Filter::default()
                    .with_adapter(self)
                    .with_type(FilterType::User)
                    .with_details(
                        format!("{}_state_new", self.get_name()),
                        Filter::filter_expression(self.get_name(), 
                            "with_state(new)"
                        )
                    );

                let filter_tag_doc = Filter::default()
                .with_adapter(self)
                .with_type(FilterType::User)
                .with_details(
                    format!("{}_tag_doc", self.get_name()),
                    Filter::filter_expression(self.get_name(), 
                        "with_tag(documentation)"
                    )
                );

                let filter_assigned_to_me = Filter::default()
                .with_adapter(self)
                .with_type(FilterType::User)
                .with_details(
                    format!("{}_assigned_to_me", self.get_name()),
                    Filter::filter_expression(self.get_name(), 
                        "assigned_to(::me)"
                    )
                );

                let filter_example_1 = Filter::default()
                .with_adapter(self)
                .with_type(FilterType::User)
                .with_details(
                    format!("{}_example_1", self.get_name()),
                    Filter::filter_expression(self.get_name(), 
                        "with_state(new)
                        with_state(open)"
                    )
                );

                let filter_example_2 = Filter::default()
                .with_adapter(self)
                .with_type(FilterType::User)
                .with_details(
                    format!("{}_example_2", self.get_name()),
                    Filter::filter_expression(self.get_name(), 
                        "in_bucket(default.bucket)
                        with_tag(documentation);;
                        in_bucket(empty.bucket)
                        with_tag(documentation)"
                    )
                );

                let filter_example_3 = Filter::default()
                .with_adapter(self)
                .with_type(FilterType::User)
                .with_details(
                    format!("{}_example_3", self.get_name()),
                    Filter::filter_expression(self.get_name(), 
                        "assigned_to(::me)
                        due_in_days(7)"
                    )
                );

                match self.filter_write(&filter_state_new)
                 .and(self.filter_write(&filter_tag_doc))
                 .and(self.filter_write(&filter_assigned_to_me))
                 .and(self.filter_write(&filter_example_1))
                 .and(self.filter_write(&filter_example_2))
                 .and(self.filter_write(&filter_example_3)) {
                    Ok(_) => (),
                    Err(err) => println!("Wasn't able to write filters as default data due to {err}"),
                }

            }

            if ticket_tables && tagticket_tables {

                let ticket_example_task = self.ticket_write(
                    &Ticket::default()
                        .with_adapter(self)
                        .with_bucket(&bucket_default)
                        .with_details(0, String::from("Example Task"), 
                            String::from("This is an example Task, created to test functionality."))
                        .with_state(&state_new)
                        .with_tags(vec![
                            &tag_low_prio,
                            &tag_documentation,
                            &Tag::default().with_name(String::from("example")).with_random_colors(),
                            &Tag::default().with_name(String::from("example2")).with_random_colors(),
                            &Tag::default().with_name(String::from("example3")).with_random_colors()
                        ])
                        .with_assignee("biochemist".to_string())
                );

                let ticket_second_task = self.ticket_write(
                    &Ticket::default()
                        .with_adapter(self)
                        .with_bucket(&bucket_default)
                        .with_details(0, String::from("Second Task"), 
                            String::from("This is the second example Task, created to test functionality."))
                        .with_state(&state_open)
                        .with_tags(vec![
                            &Tag::default().with_name(String::from("example")).with_random_colors(),
                            &Tag::default().with_name(String::from("example2")).with_random_colors(),
                            &Tag::default().with_name(String::from("example3")).with_random_colors()
                        ])
                        .with_assignee("user2".to_string())
                );

                let ticket_long_title = self.ticket_write(
                    &Ticket::default()
                        .with_adapter(self)
                        .with_bucket(&bucket_empty)
                        .with_details(0, ["The title of this ticket is very long, ",
                            "to test the extreme case of rendering a Ticket."].join(""), 
                            ["This is the third example Task, created to test rendering extremes. ",
                            "That also means, that this text is very long, so you need to read a",
                            " moment, and it also won't fit into the ticket completely. Well",
                            ", atleast thats the plan."].join(""))
                        .with_state(&state_open)
                        .with_tags(vec![
                            &tag_blocker,
                            &tag_bug,
                            &tag_documentation,
                            &Tag::default().with_name(String::from("example3")).with_random_colors()
                        ])
                );

                let ticket_markdown = self.ticket_write(
                    &Ticket::default()
                        .with_adapter(self)
                        .with_bucket(&bucket_empty)
                        .with_details(0, "This ticket contains some Markdown in the Description".to_string(), 
                            ["Here's a numbered list:",
                            "",
                            " 1. first item",
                            " 2. second item",
                            " 3. third item",
                            "",
                            "Note again how the actual text starts at 4 columns in (4 characters",
                            "from the left side). Here's a code sample:",
                            "",
                            "   # Let me re-iterate ...",
                            "   for i in 1 .. 10 { do-something(i) }",
                            "",
                            "As you probably guessed, indented 4 spaces. By the way, instead of",
                            "indenting the block, you can use delimited blocks, if you like:",
                            "",
                            "~~~",
                            "define foobar() {",
                            "   print \"Welcome to flavor country!\";",
                            "}",
                            "~~~",
                            "",
                            "(which makes copying & pasting easier). You can optionally mark the",
                            "delimited block for Pandoc to syntax highlight it:",
                            "",
                            "~~~py",
                            "import time",
                            "# Quick, count to ten!",
                            "for i in range(10):",
                            "   # (but not *too* quick)",
                            "   time.sleep(0.5)",
                            "   print(i)",
                            "~~~",
                            "",
                            "Here's a link to [the cargo website for testing](https://crates.io/)",
                            "",
                            "![image example](https://github.com/TheBiochemic/free_px_assets/blob/main/biocraft_textures/Block_Exclusive.png?raw=true \"example image\")"
                        ].join("\n"))
                        .with_state(&state_live)
                        .with_tags(vec![
                            &tag_enhancement,
                            &tag_documentation,
                            &Tag::default().with_name(String::from("markdown")).with_random_colors()
                        ])
                );

                match ticket_example_task
                 .and(ticket_second_task)
                 .and(ticket_long_title)
                 .and(ticket_markdown) {
                    Ok(_) => (),
                    Err(err) => println!("Wasn't able to write tickets as default data due to {err}"),
                }
            }
        }
    }

    fn list_builtin_filters(&self) -> Vec<Filter> {

        let buckets = self.bucket_list_all();

        buckets.iter().map(|bucket| {
            Filter::default()
                .with_details(
                    bucket.name.clone(), 
                    Filter::filter_expression(self.get_name(), &format!("in_bucket({})", bucket.name)))
                .with_type(FilterType::Bucket(bucket.identifier.id))
                .with_adapter(self)
        }).collect::<Vec<Filter>>()
    }

}