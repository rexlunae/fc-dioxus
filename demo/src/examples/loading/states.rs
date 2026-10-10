use fc_dioxus::prelude::*;

// example:start
/// Skeletons in a busy region, native progress, and progress rings.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    let names = [
        "Loading message",
        "Upload",
        "Indexing",
        "Quota",
        "Context used",
        "Done",
        "Upload failed",
    ]
    .map(AccessibleName::try_from);
    let [Ok(message), Ok(upload), Ok(indexing), Ok(quota), Ok(context), Ok(done), Ok(failed)] =
        names
    else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            Loading { label: message, class: CLUSTER.block(),
                Skeleton { shape: SkeletonShape::Circle }
                span { Skeleton { shape: SkeletonShape::Text } Skeleton { shape: SkeletonShape::Text } }
            }
            Skeleton { shape: SkeletonShape::Block }
            Progress { label: upload.clone(), value: ProgressValue::Of { value: 64.0, max: 100.0 } }
            Progress { label: indexing.clone(), value: ProgressValue::Indeterminate }
            Progress { label: quota, value: ProgressValue::Of { value: 20.0, max: 100.0 }, small: true }
            div { class: CLUSTER.block(),
                ProgressRing { label: context, percent: Percent::new(30.0), size: RingSize::Sm }
                ProgressRing { label: upload, percent: Percent::new(0.0), show_label: true }
                ProgressRing { label: indexing, percent: Percent::new(64.0), size: RingSize::Lg, show_label: true }
                ProgressRing { label: done, percent: Percent::new(100.0), size: RingSize::Lg, show_label: true }
                ProgressRing { label: failed, percent: Percent::new(45.0), size: RingSize::Lg, show_label: true, error: true }
            }
        }
    }
}
