use crate::models::tasks::Tasks;
use crate::service::google_api::GoogleApiClient;
use crate::service::google_tasks::ApiTasks;
use anyhow;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Select};

pub struct TaskManager {
    pub client: GoogleApiClient,
}

impl TaskManager {
    pub fn list_tasks(&self, show_hidden: bool) -> anyhow::Result<()> {
        let resp = &self.client.fetch_all_tasks(show_hidden);
        match resp {
            Ok(list) => {
                &self.client.localdb.insert_tasks(list.items.clone())?;
                let mut order = 1;
                for tasks in &list.items {
                    println!("{}: {}", order, tasks);
                    order += 1;
                }
            }
            Err(err) => println!("Some error occured in fetching tasks! {}", err),
        }
        Ok(())
    }

    pub fn add_task(&self) -> anyhow::Result<()> {
        let title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Title of the task")
            .with_initial_text("task")
            .allow_empty(false)
            .interact_text()?;
        let notes: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Note for task")
            .with_initial_text("note")
            .allow_empty(true)
            .interact_text()?;
        let items = vec!["No", "Yes"];
        let completed = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Is the task completed?")
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())?
            .unwrap();
        let status = if completed == 0 {
            String::from("needsAction")
        } else {
            String::from("completed")
        };
        let task = Tasks::new(None, title, notes, status);
        let resp = &self.client.add_task(task);
        match resp {
            Ok(task) => println!("Task {} has been created!", task.title),
            Err(err) => println!("Some error hass occured! {}", err),
        }
        Ok(())
    }

    pub fn show_task(&self, pos: usize) -> anyhow::Result<()> {
        let resp = &self.client.localdb.get_data()?;
        let task = resp.get(pos - 1).unwrap().id.as_ref().unwrap();
        let new_resp = &self.client.fetch_task(task.to_string());
        match new_resp {
            Ok(task) => println!("Task: {}", task),
            Err(err) => println!("Some error has occured! {}", err),
        }
        Ok(())
    }

    pub fn complete_task(&self, pos: usize, is_completed: bool) -> anyhow::Result<()> {
        let resp = &self.client.localdb.get_data()?;
        let mut task = resp.get(pos - 1).unwrap().clone();
        task.status = if is_completed {
            String::from("completed")
        } else {
            String::from("needsAction")
        };
        let new_resp = &self.client.update_task(task);
        match new_resp {
            Ok(task) => {
                if is_completed {
                    println!("Task {} marked as completed!", task.title)
                } else {
                    println!("Task {} marked as incomplete!", task.title)
                }
            }
            Err(err) => println!("Some error occured {}", err),
        }
        Ok(())
    }

    pub fn clear_tasks(&self) -> anyhow::Result<()> {
        let resp = &self.client.clear_completed_tasks();
        match resp {
            Ok(()) => println!("Cleared all the tasks!"),
            Err(err) => println!("Some error occured in fetching tasks! {}", err),
        }
        Ok(())
    }

    pub fn delete_task(&self, pos: usize) -> anyhow::Result<()> {
        let resp = &self.client.localdb.get_data()?;
        let task = resp.get(pos - 1).unwrap();
        let new_resp = &self
            .client
            .delete_task(task.id.as_ref().unwrap().to_string());
        match new_resp {
            Ok(_res) => println!("Task {} has been deleted!", &task.title),
            Err(err) => println!("Error deleting task {}", err),
        }
        Ok(())
    }
}