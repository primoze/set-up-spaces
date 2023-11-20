use std::thread;
use deno_runtime::deno_core::{ModuleSpecifier, resolve_url};
use deno_runtime::worker::{MainWorker, WorkerOptions};
use deno_runtime::BootstrapOptions;
use deno_runtime::permissions::{Permissions, PermissionsContainer};


fn main() {
    run_worker();
    run_worker();
}

fn run_worker() {
    let handle = thread::spawn(move || {

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .max_blocking_threads(8)
            .build()
            .unwrap();

        let local = tokio::task::LocalSet::new();
        local.block_on(&runtime, async move {
            let mut  worker = Worker::new();
            worker.init().await;
            worker.exec("main();");
            worker.run_event_loop().await;
        });

    });

    handle.join().unwrap();
}

struct Worker(ModuleSpecifier, MainWorker);

impl Worker {
    fn new() -> Self {

        let bootstrap = BootstrapOptions {
            cpu_count: 1,
            ..Default::default()
        };

        let options = WorkerOptions {
            bootstrap,
            ..Default::default()
        };

        let main_module = resolve_url(&format!(
            "file://{}/main.ts",
            std::env::current_dir().unwrap().to_str().unwrap()
        ))
            .unwrap();

        let main_worker = MainWorker::bootstrap_from_options(
            main_module.clone(),
            PermissionsContainer::new(Permissions::default()),
            options,
        );

        Self(main_module, main_worker)
    }

    async fn init(&mut self) {
        self.1.execute_main_module(&self.0).await.unwrap();
    }

    pub fn exec(&mut self, script: &str) {
        self.1.execute_script("anon", script.to_string().into()).unwrap();
    }

    pub async fn run_event_loop(&mut self) {
        self.1.run_event_loop(false).await.unwrap();
    }
}
