use lv2_host::execution::PluginExecutionContext;
use lv2_host::repository::PluginRepository;

#[test]
pub fn example_test() {
    let repository = PluginRepository::discover();
    let plugin = repository.find_plugin("http://my_plugin").unwrap();

    let mut execution_context = PluginExecutionContext::new();

    let instance = execution_context
        .instanciate(plugin, 44_100.0)
        .unwrap()
        .activate();
}
