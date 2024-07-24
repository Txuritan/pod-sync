#[cfg(debug_assertions)]
use crate::__private::FunctionDescription;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};

mod metrics;

pub use self::metrics::MetricsTracker;

pub trait TrackMetrics {
    fn set_build_info(build_info_labels: &BuildInfoLabels);
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self;
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels);
    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]);
}

pub struct AutometricsTracker {
    metrics_tracker: MetricsTracker,
}

impl TrackMetrics for AutometricsTracker {
    #[allow(unused_variables)]
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        MetricsTracker::set_build_info(build_info_labels);
    }

    #[allow(unused_variables)]
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        Self {
            metrics_tracker: MetricsTracker::start(gauge_labels),
        }
    }

    #[allow(unused_variables)]
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        self.metrics_tracker
            .finish(counter_labels, histogram_labels);
    }

    #[cfg(debug_assertions)]
    #[allow(unused_variables)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]) {
        MetricsTracker::intitialize_metrics(function_descriptions);
    }
}
