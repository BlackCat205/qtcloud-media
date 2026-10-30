import 'dart:async';
import 'dart:js_interop';
import 'dart:ui_web' as ui_web;

import 'package:flutter/material.dart';
import 'package:web/web.dart' as web;

import 'md.dart';

/// 把富文本写入剪贴板（见 web/index.html 中的同名 JS 函数）。
/// text/html 让粘贴进公众号编辑器时保留排版；text/plain 作降级。
@JS('copyRichText')
external JSPromise copyRichText(String html, String text);

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
  static const _viewType = 'qtcloud-preview';
  late final web.HTMLIFrameElement _iframe;
  final _controller = TextEditingController(text: sampleMarkdown);
  Timer? _debounce;
  String _html = '';

  @override
  void initState() {
    super.initState();
    _html = renderArticle(_controller.text);
    _iframe = web.HTMLIFrameElement()
      ..setAttribute(
          'style', 'border:0;width:100%;height:100%;background:#ffffff;')
      ..setAttribute('srcdoc', _previewDoc(_html));
    ui_web.platformViewRegistry
        .registerViewFactory(_viewType, (int viewId) => _iframe);
    _controller.addListener(_onChanged);
  }

  void _onChanged() {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 400), () {
      if (!mounted) return;
      setState(() => _html = renderArticle(_controller.text));
      _iframe.setAttribute('srcdoc', _previewDoc(_html));
    });
  }

  String _previewDoc(String html) =>
      '<!doctype html><html><head><meta charset="utf-8"></head>'
      '<body style="margin:16px;">$html</body></html>';

  Future<void> _copy() async {
    final text = _controller.text;
    await copyRichText(_html, text).toDart;
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('已复制。到公众号编辑器直接粘贴，排版会保留。')),
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
        title: const Text('量潮媒体云工作台'),
        actions: [
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
            child: HtmlElementView(viewType: _viewType),
          ),
        ],
      ),
    );
  }
}
