import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:buzz/shared/theme/theme.dart';
import 'package:buzz/shared/widgets/frosted_app_bar.dart';

void main() {
  group('Kura theme catalog entries', () {
    test('both halves are in the catalog', () {
      expect(findTheme(kuraThemeName), isNotNull);
      expect(findTheme(kuraDarkThemeName), isNotNull);
    });

    test('use the Kubo stone palette', () {
      final kura = findTheme(kuraThemeName)!;
      expect(kura.bg, const Color(0xFFFFFFFF));
      expect(kura.fg, const Color(0xFF0C0A09));
      expect(kura.displayName, 'Kura');

      final kuraDark = findTheme(kuraDarkThemeName)!;
      expect(kuraDark.bg, const Color(0xFF0C0A09));
      expect(kuraDark.fg, const Color(0xFFFAFAF9));
      expect(kuraDark.displayName, 'Kura Dark');
    });

    test('are a light/dark pair', () {
      expect(findTheme(kuraThemeName)!.isDark, isFalse);
      expect(findTheme(kuraDarkThemeName)!.isDark, isTrue);
      expect(themePairFor(kuraThemeName), kuraDarkThemeName);
      expect(themePairFor(kuraDarkThemeName), kuraThemeName);
    });

    test('appear as a single System-mode option labelled "Kura"', () {
      final paired = themeGroups().paired.map((t) => t.name);
      expect(paired, contains(kuraThemeName));
      expect(paired, isNot(contains(kuraDarkThemeName)));
      expect(pairedThemeLabel(kuraThemeName), 'Kura');
      expect(themeSelectionLabel(kuraThemeName, ThemeMode.system), 'Kura');
      expect(themeSelectionLabel(kuraDarkThemeName, ThemeMode.system), 'Kura');
    });

    test('forces neutral rendering without changing the stored accent', () {
      const storedAccent = '#ef4444';

      expect(
        effectiveAccentIndex(kuraThemeName, storedAccent),
        neutralAccentIndex,
      );
      expect(
        effectiveAccentIndex(kuraDarkThemeName, storedAccent),
        neutralAccentIndex,
      );
      expect(
        effectiveAccentIndex('github-light', storedAccent),
        accentIndexForWireValue(storedAccent),
      );
      expect(storedAccent, '#ef4444');
    });

    test('resolve across brightnesses like any other pair', () {
      final resolved = resolveSchemes(kuraThemeName, ThemeMode.system);
      expect(resolved.forcedMode, isNull);
      expect(resolved.light.brightness, Brightness.light);
      expect(resolved.dark.brightness, Brightness.dark);
      expect(resolved.lightTheme?.name, kuraThemeName);
      expect(resolved.darkTheme?.name, kuraDarkThemeName);

      expect(
        effectiveTheme(kuraThemeName, ThemeMode.dark)?.name,
        kuraDarkThemeName,
      );
      expect(
        effectiveTheme(kuraDarkThemeName, ThemeMode.light)?.name,
        kuraThemeName,
      );
    });

    test(
      'fallbacks expose the effective Kura theme for gradient selection',
      () {
        final coerced = resolveSchemes('nord', ThemeMode.light);
        expect(coerced.lightTheme?.name, kuraThemeName);
        expect(
          kuraTopSectionGradient(
            coerced.lightTheme!.name,
            coerced.light.brightness,
          ),
          isNotNull,
        );

        final unknown = resolveSchemes('not-a-theme', ThemeMode.light);
        expect(unknown.lightTheme?.name, kuraThemeName);
        expect(
          kuraTopSectionGradient(
            unknown.lightTheme!.name,
            unknown.light.brightness,
          ),
          isNotNull,
        );
      },
    );
  });

  group('kuraTopSectionGradient', () {
    test('is null for non-Kura themes', () {
      expect(kuraTopSectionGradient('github-light', Brightness.light), isNull);
      expect(kuraTopSectionGradient('nord', Brightness.dark), isNull);
    });

    test('paints top to bottom for both halves of the pair', () {
      for (final name in [kuraThemeName, kuraDarkThemeName]) {
        final gradient = kuraTopSectionGradient(name, Brightness.light);
        expect(gradient, isNotNull, reason: '$name should be gradient-backed');
        expect(gradient!.begin, Alignment.topCenter);
        expect(gradient.end, Alignment.bottomCenter);
        expect(gradient.colors, hasLength(2));
      }
    });

    test('brightness selects the stops, not the theme name', () {
      // Both halves enable the gradient, so System mode keeps it on across an
      // OS switch — the applied brightness alone decides which stops are used.
      final light = kuraTopSectionGradient(kuraThemeName, Brightness.light)!;
      final dark = kuraTopSectionGradient(kuraThemeName, Brightness.dark)!;

      expect(light.colors, isNot(dark.colors));
      expect(
        kuraTopSectionGradient(kuraDarkThemeName, Brightness.dark)!.colors,
        dark.colors,
      );
      expect(
        kuraTopSectionGradient(kuraDarkThemeName, Brightness.light)!.colors,
        light.colors,
      );
    });

    test('is opaque so the color replaces the frosted fill', () {
      for (final brightness in Brightness.values) {
        final gradient = kuraTopSectionGradient(kuraThemeName, brightness)!;
        for (final color in gradient.colors) {
          expect(color.a, 1.0);
        }
      }
    });
  });

  group('theme threading', () {
    BoxDecoration barDecoration(WidgetTester tester) {
      final container = tester
          .widgetList<Container>(
            find.descendant(
              of: find.byType(FrostedAppBar),
              matching: find.byType(Container),
            ),
          )
          .first;
      return container.decoration! as BoxDecoration;
    }

    Widget harness(ThemeData theme) => MaterialApp(
      theme: theme,
      home: Builder(
        builder: (context) => Stack(
          children: [
            FrostedAppBar(
              gradient: context.appColors.topSectionGradient,
              title: const Text('Home'),
            ),
          ],
        ),
      ),
    );

    testWidgets('AppTheme carries the gradient to the top section', (
      tester,
    ) async {
      await tester.pumpWidget(
        harness(
          AppTheme.light(
            topSectionGradient: kuraTopSectionGradient(
              kuraThemeName,
              Brightness.light,
            ),
          ),
        ),
      );

      final decoration = barDecoration(tester);
      expect(decoration.gradient, isNotNull);
      // A BoxDecoration cannot paint a color and a gradient at once.
      expect(decoration.color, isNull);
    });

    testWidgets('non-Kura themes keep the frosted surface fill', (
      tester,
    ) async {
      await tester.pumpWidget(harness(AppTheme.light()));

      final decoration = barDecoration(tester);
      expect(decoration.gradient, isNull);
      expect(decoration.color, isNotNull);
    });

    testWidgets('Kura section labels use 80% neutral foreground', (
      tester,
    ) async {
      await tester.pumpWidget(
        harness(
          AppTheme.light(
            topSectionGradient: kuraTopSectionGradient(
              kuraThemeName,
              Brightness.light,
            ),
          ),
        ),
      );

      final context = tester.element(find.text('Home'));
      expect(
        navigationSectionForeground(context),
        Colors.black.withValues(alpha: 0.8),
      );
    });

    testWidgets('navigation roles inherit non-Kura theme tokens', (
      tester,
    ) async {
      const primaryForeground = Color(0xFF123456);
      const secondaryForeground = Color(0xFF789ABC);
      const searchSurface = Color(0xFFDEF012);
      final theme = ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.purple).copyWith(
          onSurface: primaryForeground,
          onSurfaceVariant: secondaryForeground,
          surfaceContainerHighest: searchSurface,
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: theme,
          home: const Scaffold(body: SizedBox()),
        ),
      );

      final context = tester.element(find.byType(SizedBox));
      expect(navigationPrimaryForeground(context), primaryForeground);
      expect(navigationSecondaryForeground(context), secondaryForeground);
      expect(navigationSectionForeground(context), secondaryForeground);
      expect(navigationSearchSurface(context), searchSurface);
      expect(
        navigationDivider(context, 0.15),
        primaryForeground.withValues(alpha: 0.15),
      );
    });
  });

  group('isKuraTheme', () {
    test('matches only the Kura pair', () {
      expect(isKuraTheme(kuraThemeName), isTrue);
      expect(isKuraTheme(kuraDarkThemeName), isTrue);
      expect(isKuraTheme('github-light'), isFalse);
      expect(isKuraTheme(''), isFalse);
    });
  });
}
