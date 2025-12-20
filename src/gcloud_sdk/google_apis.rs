pub mod google {
    #![allow(dead_code, unused_imports, unused_variables, unexpected_cfgs)]

    pub mod api {
        include_proto!("google.api");
    }
    pub mod r#type {
        include_proto!("google.r#type");
    }
    pub mod rpc {
        include_proto!("google.rpc");
    }
    pub mod monitoring {
        // pub mod dashboard {
        //     pub mod v1 {
        //         #[cfg(any(feature = "google-monitoring-dashboard-v1"))]
        //         include_proto!("google.monitoring.dashboard.v1");
        //     }
        // }
        // pub mod metricsscope {
        //     pub mod v1 {
        //         #[cfg(any(feature = "google-monitoring-metricsscope-v1"))]
        //         include_proto!("google.monitoring.metricsscope.v1");
        //     }
        // }

        pub mod v3 {
            // #[cfg(any(feature = "google-monitoring-v3"))]
            include_proto!("google.monitoring.v3");
        }
    }
}
