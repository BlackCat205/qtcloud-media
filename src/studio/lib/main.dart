import 'dart:async';

import 'package:flutter/material.dart';

import 'copy.dart'
    if (dart.library.js_interop) 'copy_web.dart'
    if (dart.library.io) 'copy_desktop.dart';
import 'drafts.dart'
    if (dart.library.js_interop) 'drafts_web.dart'
    if (dart.library.io) 'drafts_io.dart';
import 'md.dart';
import 'preview.dart'
    if (dart.library.js_interop) 'preview_web.dart'
    if (dart.library.io) 'preview_desktop.dart';
import 'remote.dart'
    if (dart.library.js_interop) 'remote_web.dart'
    if (dart.library.io) 'remote_io.dart';
import 'settings.dart';
import 'theme.dart';
import 'theme_loader.dart'
    if (dart.library.js_interop) 'theme_loader_web.dart'
    if (dart.library.io) 'theme_loader_io.dart';

void main() {
  runApp(const StudioApp());
}

const sampleMarkdown = '''# 标题写在这里（不进正文）

一条命令，把 Markdown 文章排好版，直接送进微信公众号草稿箱。

## 小节

正文段落，支持 **加粗**、*斜体*、`行内代码` 与 [链接](https://example.com)。

- 列表项一
- 列表项二

```bash
qtcloud-media render article.md --theme themes/default.toml -o article.html
```

> 引用一段话。

写完点右上角「复制到公众号」，到公众号编辑器里直接粘贴即可。
''';

class StudioApp extends StatelessWidget {
  const StudioApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '量潮媒体云',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF07C160)),
        useMaterial3: true,
      ),
      home: const StudioShell(),
    );
  }
}

/// 侧边导航壳：文章（现编辑页）/ 图文、视频（预留）/ 设置。
class StudioShell extends StatefulWidget {
  const StudioShell({super.key});

  @override
  State<StudioShell> createState() => _StudioShellState();
}

class _StudioShellState extends State<StudioShell> {
  int _index = 0;
  bool _settings = false;
  final _articleKey = GlobalKey<_ArticlePageState>();

  void _select(int i) {
    setState(() {
      _index = i;
      _settings = false;
    });
    // 从设置页切回文章页时重读服务端连接设置，保证推送用最新值
    if (i == 0) _articleKey.currentState?.reloadRemote();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          NavigationRail(
            selectedIndex: _settings ? null : _index,
            onDestinationSelected: _select,
            labelType: NavigationRailLabelType.all,
            destinations: const [
              NavigationRailDestination(
                icon: Icon(Icons.article_outlined),
                label: Text('文章'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.image_outlined),
                label: Text('图文'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.videocam_outlined),
                label: Text('视频'),
              ),
            ],
            trailing: Expanded(
              child: Align(
                alignment: Alignment.bottomCenter,
                child: Padding(
                  padding: const EdgeInsets.only(bottom: 12),
                  child: IconButton(
                    tooltip: '设置',
                    onPressed: () => setState(() => _settings = true),
                    icon: Icon(
                      Icons.settings_outlined,
                      color: _settings
                          ? Theme.of(context).colorScheme.primary
                          : null,
                    ),
                  ),
                ),
              ),
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: IndexedStack(
              index: _settings ? 3 : _index,
              children: [
                ArticlePage(key: _articleKey),
                const PlaceholderPage(title: '图文'),
                const PlaceholderPage(title: '视频'),
                const SettingsPage(),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// 预留分区页：图文、视频尚未开发。
class PlaceholderPage extends StatelessWidget {
  const PlaceholderPage({super.key, required this.title});

  final String title;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(title)),
      body: const Center(child: Text('规划中')),
    );
  }
}

class ArticlePage extends StatefulWidget {
  const ArticlePage({super.key});

  @override
  State<ArticlePage> createState() => _ArticlePageState();
}

class _ArticlePageState extends State<ArticlePage> {
  final _controller = TextEditingController(text: sampleMarkdown);
  Timer? _debounce;
  List<Block> _blocks = const [];
  ThemeStyles _theme = const ThemeStyles();
  String _html = '';
  String? _draftName;
  List<String> _draftNames = const [];
  RemoteSettings? _remote;

  @override
  void initState() {
    super.initState();
    _reparse();
    _controller.addListener(_onChanged);
    _loadTheme();
    _refreshDrafts();
    _loadRemote();
  }

  Future<void> _loadRemote() async {
    final s = await loadSettings();
    if (!mounted) return;
    setState(() => _remote = s);
  }

  /// 供导航壳调用：从设置页切回文章页时重读服务端连接。
  void reloadRemote() => _loadRemote();

  Future<void> _refreshDrafts() async {
    final names = await listDrafts();
    if (!mounted) return;
    setState(() => _draftNames = names);
  }

  /// 主题：桌面端读 XDG（~/.local/share|~/.config/qtcloud-media/themes/），
  /// 没有再回退内置默认；Web 端直接内置默认。
  Future<void> _loadTheme() async {
    final theme = await loadThemeStyles();
    if (!mounted) return;
    setState(() {
      _theme = theme;
      _reparse();
    });
  }

  void _reparse() {
    _blocks = parseMarkdown(_controller.text);
    _html = articleHtml(_blocks, _theme);
  }

  void _onChanged() {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 400), () {
      if (!mounted) return;
      setState(_reparse);
    });
  }

  Future<void> _publishRemote() async {
    final r = _remote;
    if (r == null) return;
    final title = _draftName ?? '未命名';
    try {
      final result = await publishRemote(
        provider: r.provider,
        account: r.account,
        markdown: _controller.text,
        title: title,
        draft: _draftName,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('已推送：$result（不会自动群发）')));
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('远程推送失败：$e')));
    }
  }

  Future<void> _copy() async {
    await copyRich(_html, _controller.text);
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('已复制。到公众号编辑器直接粘贴，排版会保留。')));
  }

  Future<void> _openDraft(String name) async {
    final content = await readDraft(name);
    if (!mounted) return;
    if (content == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('草稿读取失败：$name')));
      return;
    }
    setState(() {
      _draftName = name;
      _controller.text = content;
      _reparse();
    });
  }

  Future<void> _saveDraftAs() async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('保存为草稿'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: '草稿名'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, controller.text.trim()),
            child: const Text('保存'),
          ),
        ],
      ),
    );
    if (name == null || name.isEmpty || !mounted) return;
    await saveDraft(name, _controller.text);
    if (!mounted) return;
    setState(() => _draftName = name);
    await _refreshDrafts();
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('已保存草稿「$name」。CLI 用草稿名 publish 即可接力。')),
    );
  }

  Future<void> _saveCurrentDraft() async {
    final name = _draftName;
    if (name == null) return;
    await saveDraft(name, _controller.text);
    if (!mounted) return;
    await _refreshDrafts();
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('已保存草稿「$name」。CLI 用草稿名 publish 即可接力。')),
    );
  }

  /// 草稿清单面板：导航栏与编辑器之间，点项打开、＋存为新草稿、保存写回当前草稿。
  Widget _buildDraftPanel() {
    return Container(
      width: 240,
      color: Theme.of(context).colorScheme.surfaceContainerLow,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            child: Row(
              children: [
                const SizedBox(width: 8),
                Text('草稿', style: Theme.of(context).textTheme.titleSmall),
                const Spacer(),
                IconButton(
                  tooltip: '保存为新草稿',
                  onPressed: _saveDraftAs,
                  icon: const Icon(Icons.add),
                  visualDensity: VisualDensity.compact,
                ),
                IconButton(
                  tooltip: '保存当前草稿',
                  onPressed: _draftName == null ? null : _saveCurrentDraft,
                  icon: const Icon(Icons.save),
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: _draftNames.isEmpty
                ? const Center(child: Text('（暂无草稿）'))
                : ListView.builder(
                    itemCount: _draftNames.length,
                    itemBuilder: (context, i) {
                      final n = _draftNames[i];
                      return ListTile(
                        dense: true,
                        selected: n == _draftName,
                        title: Text(n, overflow: TextOverflow.ellipsis),
                        onTap: () => _openDraft(n),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildDraftPanel(),
          const VerticalDivider(width: 1),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                AppBar(
                  title: Text(
                    _draftName == null ? '量潮媒体云' : '量潮媒体云 · $_draftName',
                  ),
                  actions: [
                    if (_remote != null)
                      IconButton(
                        tooltip: '推到公众号（远程：${_remote!.account}）',
                        onPressed: _publishRemote,
                        icon: const Icon(Icons.cloud_upload),
                      ),
                    const SizedBox(width: 4),
                    FilledButton.icon(
                      onPressed: _copy,
                      icon: const Icon(Icons.copy),
                      label: const Text('复制到公众号'),
                    ),
                    const SizedBox(width: 16),
                  ],
                ),
                Expanded(
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Expanded(
                        child: Padding(
                          padding: const EdgeInsets.all(12),
                          child: TextField(
                            controller: _controller,
                            maxLines: null,
                            expands: true,
                            decoration: const InputDecoration(
                              border: OutlineInputBorder(),
                              labelText: 'Markdown',
                            ),
                            style: const TextStyle(
                              fontFamily: 'monospace',
                              fontSize: 14,
                            ),
                          ),
                        ),
                      ),
                      const VerticalDivider(width: 1),
                      Expanded(
                        child: ClipRect(child: buildPreview(_blocks, _theme)),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
