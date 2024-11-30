use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

struct TodoList {
    tasks: Vec<Task>,
    next_id: usize,
}

impl TodoList {
    fn new() -> Self { 
        TodoList {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    fn add_task(&mut self, description: String) -> Result<usize, String> {
        // More robust task description validation
        let trimmed_desc = description.trim();
        if trimmed_desc.is_empty() {
            return Err("Task description cannot be empty".to_string());
        }

        let task = Task {
            id: self.next_id,
            description: trimmed_desc.to_string(),
            completed: false,
        };
        self.tasks.push(task);
        let task_id = self.next_id;
        self.next_id += 1;
        Ok(task_id)
    }

    fn list_tasks(&self) {
        if self.tasks.is_empty() {
            println!("No tasks found.");
            return;
        }
    
        for (_index, task) in self.tasks.iter().enumerate() {
            let status = if task.completed { "[x]" } else { "[ ]" };
            println!("{} ID: {}, {}", status, task.id, task.description);
        }
    }

    fn complete_task(&mut self, id: usize) -> Result<(), String> {
        self.tasks.iter_mut()
            .find(|task| task.id == id)
            .map(|task| {
                task.completed = true;
                Ok(())
            })
            .unwrap_or_else(|| Err(format!("Task with ID {} not found", id)))
    }

    fn remove_task(&mut self, id: usize) -> Result<(), String> {
        self.tasks.iter()
            .position(|task| task.id == id)
            .map(|index| {
                self.tasks.remove(index);
                Ok(())
            })
            .unwrap_or_else(|| Err(format!("Task with ID {} not found", id)))
    }

    fn save_tasks(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for task in &self.tasks {
            let status = if task.completed { "completed" } else { "pending" };
            // Escape commas in description to prevent CSV parsing issues
            let escaped_desc = task.description.replace(',', "\\,");
            writeln!(file, "{},{},{}", task.id, status, escaped_desc)?;
        }
        Ok(())
    }

    fn load_tasks(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut todo_list = TodoList::new();
        
        if !Path::new(filename).exists() {
            return Ok(todo_list);
        }

        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 3 {
                let id = parts[0].parse().map_err(|_| 
                    format!("Invalid ID in line {}", line_num + 1)
                )?;
                let completed = parts[1] == "completed";
                // Unescape commas if needed
                let description = parts[2].replace("\\,", ",");

                todo_list.tasks.push(Task {
                    id,
                    description,
                    completed,
                });
            }
        }

        // Update next_id to be higher than existing task IDs
        if let Some(max_id) = todo_list.tasks.iter().map(|task| task.id).max() {
            todo_list.next_id = max_id + 1;
        }

        Ok(todo_list)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "todo_list.txt";
    let mut todo_list = TodoList::load_tasks(filename)?;

    loop {
        println!("\nTodo List Manager");
        println!("1. Add Task");
        println!("2. List Tasks");
        println!("3. Complete Task");
        println!("4. Remove Task");
        println!("5. Save and Exit");

        print!("Enter your choice: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice: u32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input. Please enter a number.");
                continue;
            }
        };

        match choice {
            1 => {
                print!("Enter task description: ");
                io::stdout().flush()?;
                let mut description = String::new();
                io::stdin().read_line(&mut description)?;
                
                match todo_list.add_task(description) {
                    Ok(task_id) => println!("Task added with ID: {}", task_id),
                    Err(e) => println!("Error: {}", e),
                }
            }
            2 => todo_list.list_tasks(),
            3 => {
                print!("Enter task ID to complete: ");
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                
                match id_str.trim().parse() {
                    Ok(id) => {
                        match todo_list.complete_task(id) {
                            Ok(_) => println!("Task {} completed", id),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    Err(_) => println!("Invalid task ID"),
                }
            }
            4 => {
                print!("Enter task ID to remove: ");
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                
                match id_str.trim().parse() {
                    Ok(id) => {
                        match todo_list.remove_task(id) {
                            Ok(_) => println!("Task {} removed", id),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    Err(_) => println!("Invalid task ID"),
                }
            }
            5 => {
                todo_list.save_tasks(filename)?;
                println!("Tasks saved. Goodbye!");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }

    Ok(())
}