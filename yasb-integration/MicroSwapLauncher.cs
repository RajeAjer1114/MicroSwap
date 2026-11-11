using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

// MicroSwap v1.0.1 exposes its carousel through global-hotkey's message window.
// Send its existing Alt+W registration directly, avoiding process restarts and
// avoiding synthetic keyboard input to whichever application has focus.
internal static class MicroSwapLauncher
{
    private delegate bool EnumWindowProc(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowProc callback, IntPtr parameter);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetClassName(IntPtr window, StringBuilder text, int count);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, StringBuilder text, int count);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] private static extern bool PostMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] private static extern bool AllowSetForegroundWindow(uint processId);
    private const uint HotkeyMessage = 0x0312;
    private const uint AltWId = (1u << 16) | 41u;

    private static Process FindApp(string executable)
    {
        foreach (Process process in Process.GetProcessesByName("MicroSwap"))
        {
            try { if (String.Equals(process.MainModule.FileName, executable, StringComparison.OrdinalIgnoreCase)) return process; }
            catch { }
            process.Dispose();
        }
        return null;
    }

    private static IntPtr FindHotkeyWindow(int processId)
    {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr window, IntPtr unused) {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            if (owner != (uint)processId) return true;
            StringBuilder name = new StringBuilder(256);
            GetClassName(window, name, name.Capacity);
            if (name.ToString() != "global_hotkey_app") return true;
            found = window;
            return false;
        }, IntPtr.Zero);
        return found;
    }

    private static void Status(Process app)
    {
        if (app == null) { Console.WriteLine("not running"); return; }
        Console.WriteLine("pid=" + app.Id + " hotkeyWindow=" + FindHotkeyWindow(app.Id));
        EnumWindows(delegate(IntPtr window, IntPtr unused) {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            if (owner != (uint)app.Id) return true;
            StringBuilder title = new StringBuilder(256);
            GetWindowText(window, title, title.Capacity);
            if (title.ToString() == "MicroSwap") Console.WriteLine("carouselVisible=" + IsWindowVisible(window));
            return true;
        }, IntPtr.Zero);
    }

    [STAThread]
    private static int Main(string[] args)
    {
        string directory = AppDomain.CurrentDomain.BaseDirectory;
        string executable = Path.Combine(directory, "MicroSwap.exe");
        bool created;
        using (Mutex mutex = new Mutex(true, "Local\\MicroSwapWidgetLauncher", out created))
        {
            if (!created) return 0;
            try
            {
                using (Process app = FindApp(executable))
                {
                    if (args.Length > 0 && args[0] == "--status") { Status(app); return 0; }
                    if (app == null)
                    {
                        Process.Start(new ProcessStartInfo(executable) { WorkingDirectory = directory, UseShellExecute = false });
                        return 0;
                    }
                    IntPtr receiver = FindHotkeyWindow(app.Id);
                    if (receiver == IntPtr.Zero) throw new InvalidOperationException("MicroSwap's hotkey receiver is not ready; no duplicate instance was started.");
                    AllowSetForegroundWindow((uint)app.Id);
                    if (!PostMessage(receiver, HotkeyMessage, new UIntPtr(AltWId), new IntPtr((0x57 << 16) | 1)))
                        throw new InvalidOperationException("Could not deliver the carousel toggle.");
                    Thread.Sleep(200);
                    return 0;
                }
            }
            catch (Exception error)
            {
                File.AppendAllText(Path.Combine(directory, "launcher.log"), DateTime.Now.ToString("s") + " " + error.Message + Environment.NewLine);
                return 1;
            }
            finally { mutex.ReleaseMutex(); }
        }
    }
}
