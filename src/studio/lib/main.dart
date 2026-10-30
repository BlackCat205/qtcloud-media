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
      title: '量潮媒体云工作台',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF07C160)),
        useMaterial3: true,
      ),
      home: const StudioPage(),
    );
  }
}

class StudioPage extends StatefulWidget {
  const StudioPage({super.key});

  @override
  State<StudioPage> createState() => _StudioPageState();
}

class _StudioPageState extends State<StudioPage> {
  final _controller = TextEditingController(text: sampleMarkdown);
  Timer? _debounce;
  List<Block> _blocks = const [];
  ThemeStyles _theme = const ThemeStyles();
  String _html = '';
  String? _draftName;
  List<String> _draftNames = const [];

  @override
  void initState() {
    super.initState();
    _reparse();
    _controller.addListener(_onChanged);
    _loadTheme();
    _refreshDrafts();
  }

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

  Future<void> _copy() async {
    await copyRich(_html, _controller.text);
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('已复制。到公众号编辑器直接粘贴，排版会保留。')),
    );
  }

  Future<void> _openDraft(String name) async {
    final content = await readDraft(name);
    if (!mounted) return;
    if (content == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('草稿读取失败：$name')),
      );
      return;
    }
    setState(() {
      _draftName = name;
      _controller.text = content;
      _reparse();
    });
  }

  Future<void> _saveDraft() async {
    var name = _draftName;
    if (name == null) {
      final controller = TextEditingController();
      name = await showDialog<String>(
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
                onPressed: () => Navigator.pop(context), child: const Text('取消')),
            FilledButton(
                onPressed: () => Navigator.pop(context, controller.text.trim()),
                child: const Text('保存')),
          ],
        ),
      );
      if (name == null || name.isEmpty || !mounted) return;
    }
    await saveDraft(name, _controller.text);
    if (!mounted) return;
    setState(() => _draftName = name);
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('已保存草稿「$name」。CLI 用草稿名 publish 即可接力。')),
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
      appBar: AppBar(
        title: Text(_draftName == null
            ? '量潮媒体云工作台'
            : '量潮媒体云工作台 · $_draftName'),
        actions: [
          PopupMenuButton<String>(
            tooltip: '打开草稿',
            icon: const Icon(Icons.folder_open),
            onSelected: _openDraft,
            onOpened: _refreshDrafts,
            itemBuilder: (context) => _draftNames.isEmpty
                ? const [PopupMenuItem<String>(enabled: false, child: Text('（暂无草稿）'))]
                : _draftNames
                    .map((n) => PopupMenuItem(value: n, child: Text(n)))
                    .toList(),
          ),
          IconButton(
            tooltip: '保存草稿',
            onPressed: _saveDraft,
            icon: const Icon(Icons.save),
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
      body: Row(
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
                style: const TextStyle(fontFamily: 'monospace', fontSize: 14),
              ),
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: ClipRect(child: buildPreview(_blocks, _theme)),
          ),
        ],
      ),
    );
  }
}
