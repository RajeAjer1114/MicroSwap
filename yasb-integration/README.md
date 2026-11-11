# MicroSwap launcher for YASB

Click the YASB Wallpapers widget to start MicroSwap or toggle the carousel if MicroSwap is already running. Right-click changes the wallpaper through YASB.

## Setup

1. Install [MicroSwap](https://github.com/SUPERL0SER256/MicroSwap) and [YASB](https://github.com/amnweb/yasb). Check that `Alt+W` opens the MicroSwap carousel.
2. In PowerShell, open this folder and compile the launcher using the Windows .NET Framework C# compiler.

   ```powershell
   & "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe" /nologo /target:winexe /out:MicroSwapLauncher.exe MicroSwapLauncher.cs
   ```

3. Put `MicroSwapLauncher.exe` in the same folder as `MicroSwap.exe`. If MicroSwap is already running, make sure it was started from that folder.
4. In your YASB `config.yaml`, merge the widget from [yasb.example.yaml](yasb.example.yaml) into your existing top-level `widgets` section. If you already have a Wallpapers widget, copy only the `on_left` action into its `options.callbacks` and keep your existing settings.
5. Replace the example paths with your wallpaper folder and launcher location. If you added a new widget, add its name to the bar's left, center, or right widget list. Reload YASB if it does not pick up the changes automatically.

## Compatibility

The launcher uses MicroSwap v1.0.1's Windows hotkey window and its default `Alt+W` registration. A MicroSwap release that changes either may need a launcher update. Run MicroSwap and YASB at the same Windows privilege level. If a click fails, check `launcher.log` beside the launcher.
