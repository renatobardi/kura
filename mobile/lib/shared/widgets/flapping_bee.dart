import 'dart:math' show min;

import 'package:flutter/material.dart';

/// The Kura mark: a vermilion hanko seal carrying the kanji 蔵.
///
/// The class keeps the historical `FlappingBee` name and parameters so the
/// tap-to-press mark ([TappableFlappingBee]) and the pull-to-refresh
/// indicator ([BeeRefreshIndicator]) keep driving it unchanged; the file and
/// class are renamed in the phase-2 internals rebrand.
class FlappingBee extends StatelessWidget {
  /// The rendered width of the mark.
  ///
  /// Height follows the legacy 466:309 box so existing layouts keep their
  /// footprint; the seal itself is square and centered inside that box.
  final double width;

  /// The color used for the seal body.
  final Color color;

  /// How hard the seal is pressed, from 0 to 1.
  ///
  /// 0 renders the seal at rest; 1 renders it slightly compressed. Callers
  /// animate this for the tap and pull-to-refresh treatments.
  final double flapAmount;

  /// How far the glyph has faded in, from 0 to 1, or null for the full glyph.
  ///
  /// Only the pull-to-refresh treatment sets this.
  final double? eyeProgress;

  const FlappingBee({
    required this.width,
    required this.color,
    required this.flapAmount,
    this.eyeProgress,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return RepaintBoundary(
      child: CustomPaint(
        size: Size(width, width * 309 / 466),
        painter: _KuraSealPainter(
          color: color,
          pressAmount: flapAmount,
          glyphProgress: eyeProgress,
        ),
      ),
    );
  }
}

class _KuraSealPainter extends CustomPainter {
  final Color color;
  final double pressAmount;
  final double? glyphProgress;

  const _KuraSealPainter({
    required this.color,
    required this.pressAmount,
    this.glyphProgress,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final side = min(size.width, size.height) * (1 - 0.06 * pressAmount);
    final rect = Rect.fromCenter(
      center: Offset(size.width / 2, size.height / 2),
      width: side,
      height: side,
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, Radius.circular(side * 0.18)),
      Paint()..color = color,
    );

    final glyphAlpha = (glyphProgress ?? 1.0).clamp(0.0, 1.0);
    final brightness = ThemeData.estimateBrightnessForColor(color);
    final glyphColor = brightness == Brightness.dark
        ? const Color(0xFFF7F4EE)
        : const Color(0xFF1C1A17);
    final painter = TextPainter(
      text: TextSpan(
        text: '蔵',
        style: TextStyle(
          color: glyphColor.withValues(alpha: glyphAlpha),
          fontSize: side * 0.62,
          fontWeight: FontWeight.w500,
          height: 1,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(
      canvas,
      Offset(
        rect.center.dx - painter.width / 2,
        rect.center.dy - painter.height / 2,
      ),
    );
  }

  @override
  bool shouldRepaint(_KuraSealPainter oldDelegate) =>
      color != oldDelegate.color ||
      pressAmount != oldDelegate.pressAmount ||
      glyphProgress != oldDelegate.glyphProgress;
}
