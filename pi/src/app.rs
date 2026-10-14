mod api;

use topcoat::router::{RouterBuilder, RouterBuilderDiscoverExt, module_router};

pub fn router() -> RouterBuilder {
    module_router!().discover()
}
