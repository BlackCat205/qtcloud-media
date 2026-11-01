import 'dart:ui_web' as ui_web;

import 'package:flutter/material.dart';
import 'package:web/web.dart' as web;

import 'md.dart';
import 'theme.dart';

final _iframe = web.HTMLIFrameElement()
  ..setAttribute(
    'style',
    'border:0;width:100%;height:100%;background:#ffffff;',
  );
var _registered = false;

Widget buildPreview(
  List<Block> blocks,
  ThemeStyles theme, {
  String? title,
}) {
  if (!_registered) {
    _registered = true;
    ui_web.platformViewRegistry.registerViewFactory(
      'qtcloud-preview',
      (int viewId) => _iframe,
    );
  }
  _iframe.setAttribute(
    'srcdoc',
    '<!doctype html><html><head><meta charset="utf-8"></head>'
        '<body style="margin:16px;">${articleHtml(blocks, theme, title: title)}</body></html>',
  );
  return const HtmlElementView(viewType: 'qtcloud-preview');
}
