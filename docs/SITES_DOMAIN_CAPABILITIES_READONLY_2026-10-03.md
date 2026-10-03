# Sites 域名能力只读核对

父线程通报现有域名访问被云浏览器自动审批拒绝，与用户先前的域名隐私要求冲突。本执行者不访问该站点，不使用新URL、代理或其他浏览器绕过；此前Site15两UI修复的父线程亲测结果发生于拒绝前，保持其原有证据边界。

仅检查当前官方已暴露Sites工具的完整说明和schema，没有调用站点/域名变更接口，没有读取DNS或绑定域名：

- `sites_change_site_slug`输入仅有project_id与slug，说明为改变站点的公共URL标签；没有修改账号子域的参数。当前暴露工具中未发现账号子域改名能力，不能声称可通过改站点slug去掉账号标识。
- `sites_add_custom_domain`接受现有project_id及裸hostname，为已发布Site添加自定义域名；响应包括子域CNAME目标、主域A记录目标、App Garden/Cloudflare验证记录及pending/active/failed状态。仅证明有绑定能力；还需要用户提供可管理的域名及DNS记录配置，当前没有此信息。
- `sites_list_custom_domains`和`sites_refresh_custom_domain_status`可查看绑定和验证状态。本次连这两个只读接口也未调用，因为任务仅要求查官方能力。

完整原生工具说明/schema保存在本次私有 `domain-capabilities.json`。未改名、迁移、新建替换Site、买域名、继续体验或重新部署。父线程自行处理用户的下一步域名/访问授权；本地卡机制工作继续。
