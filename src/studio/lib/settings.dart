/// 设置页：服务端连接（provider 地址 + 账号），原文章页 AppBar 的远程设置弹窗移入此处。
library;

import 'package:flutter/material.dart';

import 'remote.dart'
    if (dart.library.js_interop) 'remote_web.dart'
    if (dart.library.io) 'remote_io.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  final _providerCtrl = TextEditingController();
  String? _account;
  Future<List<String>> _accounts = Future.value(const []);

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final s = await loadSettings();
    if (!mounted) return;
    setState(() {
      if (s != null) {
        _providerCtrl.text = s.provider;
        _account = s.account;
        if (s.provider.isNotEmpty) {
          _accounts = listAccounts(s.provider);
        }
      }
    });
  }

  void _refreshAccounts(String provider) {
    setState(() => _accounts = listAccounts(provider.trim()));
  }

  Future<void> _save() async {
    final s = RemoteSettings(
      provider: _providerCtrl.text.trim(),
      account: _account ?? '',
    );
    await saveSettings(s);
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('已保存服务端连接设置')));
  }

  @override
  void dispose() {
    _providerCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('设置')),
      body: Align(
        alignment: Alignment.topLeft,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: SizedBox(
            width: 480,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('服务端连接', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 12),
                TextField(
                  controller: _providerCtrl,
                  decoration: const InputDecoration(
                    labelText: 'provider 地址',
                    hintText: 'http://127.0.0.1:8787',
                  ),
                  onChanged: _refreshAccounts,
                ),
                const SizedBox(height: 12),
                FutureBuilder<List<String>>(
                  future: _accounts,
                  builder: (context, snap) => DropdownButtonFormField<String>(
                    initialValue: (snap.data?.contains(_account) ?? false)
                        ? _account
                        : null,
                    hint: Text(
                      snap.connectionState == ConnectionState.waiting
                          ? '读取账号中…'
                          : (snap.hasError)
                          ? '（读取账号失败，检查 provider 地址）'
                          : (snap.data?.isEmpty ?? true)
                          ? '（provider 侧暂无账号）'
                          : '选择账号',
                    ),
                    items: (snap.data ?? [])
                        .map((a) => DropdownMenuItem(value: a, child: Text(a)))
                        .toList(),
                    onChanged: (v) => setState(() => _account = v),
                  ),
                ),
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: _save,
                  icon: const Icon(Icons.save),
                  label: const Text('保存'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
