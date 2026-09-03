import 'package:flutter/material.dart';

import 'accent_colors.dart';
import 'app_colors.dart';

/// Name of the first-party Kura theme. Uses the Kubo stone palette; the top
/// section renders as a flat fill (no gradient, per identity guidelines) via
/// [kuraTopSectionGradient].
const kuraThemeName = 'kura';

/// Name of the dark counterpart. Paired with [kuraThemeName] in `themePairs`,
/// so the two behave as a single "Kura" choice under System mode.
const kuraDarkThemeName = 'kura-dark';

/// Whether [themeName] is either half of the Kura pair. Both halves enable the
/// gradient so System mode keeps it on across an OS light/dark switch.
bool isKuraTheme(String themeName) =>
    themeName == kuraThemeName || themeName == kuraDarkThemeName;

/// Whether the current widget tree is using the first-party Kura treatment.
bool isKuraThemeContext(BuildContext context) =>
    Theme.of(context).extension<AppColors>()?.topSectionGradient != null;

/// Primary foreground for the mobile top navigation.
///
/// Every theme uses its own [ColorScheme.onSurface]. Kura is the exception:
/// its desktop-matching top gradient needs a neutral black or white foreground
/// rather than the accent-derived color scheme foreground.
Color navigationPrimaryForeground(BuildContext context) {
  final scheme = Theme.of(context).colorScheme;
  if (!isKuraThemeContext(context)) return scheme.onSurface;
  return scheme.brightness == Brightness.dark ? Colors.white : Colors.black;
}

/// Secondary label and placeholder foreground for the mobile top navigation.
Color navigationSecondaryForeground(BuildContext context) {
  final scheme = Theme.of(context).colorScheme;
  if (!isKuraThemeContext(context)) return scheme.onSurfaceVariant;
  return navigationPrimaryForeground(context).withValues(alpha: 0.4);
}

/// Channel-section label and icon foreground for the mobile side navigation.
///
/// Section labels need more hierarchy than a placeholder. Kura therefore uses
/// a stronger neutral over its gradient, while all other themes preserve their
/// established secondary foreground token.
Color navigationSectionForeground(BuildContext context) {
  final scheme = Theme.of(context).colorScheme;
  if (!isKuraThemeContext(context)) return scheme.onSurfaceVariant;
  return navigationPrimaryForeground(context).withValues(alpha: 0.8);
}

/// Search-field surface for the mobile top navigation.
Color navigationSearchSurface(BuildContext context) {
  final scheme = Theme.of(context).colorScheme;
  if (!isKuraThemeContext(context)) return scheme.surfaceContainerHighest;
  return navigationPrimaryForeground(context).withValues(alpha: 0.04);
}

/// A low-contrast navigation divider derived from the active theme foreground.
Color navigationDivider(BuildContext context, double opacity) =>
    navigationPrimaryForeground(context).withValues(alpha: opacity);

/// Buzz renders with its fixed neutral foreground while preserving the stored
/// wire accent so the user's choice returns on another theme.
int effectiveAccentIndex(String themeName, String storedAccent) {
  if (isKuraTheme(themeName)) return neutralAccentIndex;
  return accentIndexForWireValue(storedAccent) ?? defaultAccentIndex;
}

/// Flat top-section fill for the Kura theme (washi/sumi) — no gradient, per
/// identity guidelines. Kept as matching top/bottom stops so the existing
/// [LinearGradient] plumbing in [kuraTopSectionGradient] renders as a solid
/// fill without touching call sites.
const _lightTop = Color(0xFFF7F4EE);
const _lightBottom = Color(0xFFF7F4EE);
const _darkTop = Color(0xFF151412);
const _darkBottom = Color(0xFF151412);

/// The Buzz gradient for the app's top section, or null when [themeName] is not
/// a Buzz theme — in which case the section keeps its default frosted fill.
///
/// The stops are fully opaque: under Buzz the color replaces the frosted
/// treatment rather than tinting it, matching desktop's solid sidebar canvas.
///
/// [brightness] comes from the applied color scheme rather than the theme name,
/// so System mode picks the right stops as the OS switches.
LinearGradient? kuraTopSectionGradient(
  String themeName,
  Brightness brightness,
) {
  if (!isKuraTheme(themeName)) return null;

  final isDark = brightness == Brightness.dark;
  return LinearGradient(
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
    colors: [
      isDark ? _darkTop : _lightTop,
      isDark ? _darkBottom : _lightBottom,
    ],
  );
}
