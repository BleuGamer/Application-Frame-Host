pub mod logger {
    use async_logger::AsyncLoggerNB;
    use async_logger::FileWriter;
    use std::{sync::Arc, thread};

    use std::path::PathBuf;

    struct Sink {

    }

    pub struct File {
        file: PathBuf,

        //_sink: Sink
    }

    impl File {
        pub fn new(_file: impl Into<PathBuf>) -> File {

            let file = File {
                file: _file.into(),

            };
            
            file
        }
    }
}
