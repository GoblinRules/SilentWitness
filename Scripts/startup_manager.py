#!/usr/bin/env python3
"""
SilentWitness Startup Manager
Manages startup shortcuts for SilentWitness applications
"""

import os
import sys
import shutil
import configparser
import winreg
from pathlib import Path
import logging

# Add the SilentWitness Python packages to the path
sys.path.append(r"C:\Tools\SilentWitness\Python\Lib\site-packages")

class StartupManager:
    def __init__(self, config_path=None):
        """Initialize the startup manager with configuration"""
        if config_path is None:
            # Try to find config.ini in current directory first
            current_dir = os.path.dirname(os.path.abspath(__file__))
            config_path = os.path.join(current_dir, "config.ini")
            
            # Fallback to SilentWitness path if current path doesn't exist
            if not os.path.exists(config_path):
                config_path = r"C:\Tools\SilentWitness\Scripts\config.ini"
        
        self.config_path = config_path
        self.config = configparser.ConfigParser()
        self.load_config()
        
        # Setup logging
        logging.basicConfig(level=logging.INFO)
        self.logger = logging.getLogger(__name__)
        
        # Define paths - use current directory if possible
        current_dir = os.path.dirname(os.path.abspath(__file__))
        if os.path.exists(os.path.join(current_dir, "..", "Startup")):
            self.tools_dir = os.path.dirname(current_dir)
            self.startup_dir = os.path.join(self.tools_dir, "Startup", "Recorder")
        else:
            self.tools_dir = r"C:\Tools\SilentWitness"
            self.startup_dir = r"C:\Tools\SilentWitness\Startup\Recorder"
        
    def load_config(self):
        """Load configuration from config.ini"""
        try:
            if os.path.exists(self.config_path):
                # Try to read with different encodings
                try:
                    self.config.read(self.config_path, encoding='utf-8')
                except UnicodeDecodeError:
                    self.config.read(self.config_path, encoding='utf-8-sig')
                except Exception:
                    self.config.read(self.config_path, encoding='latin-1')
            else:
                if hasattr(self, 'logger'):
                    self.logger.warning(f"Config file not found: {self.config_path}")
        except Exception as e:
            if hasattr(self, 'logger'):
                self.logger.error(f"Error loading config: {e}")
    
    def get_startup_menu_path(self):
        """Get the Start Menu Programs path for the current user"""
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, 
                               r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders") as key:
                start_menu = winreg.QueryValueEx(key, "Programs")[0]
                return start_menu
        except Exception as e:
            self.logger.error(f"Error getting Start Menu path: {e}")
            # Fallback to default path
            return os.path.expanduser(r"~\AppData\Roaming\Microsoft\Windows\Start Menu\Programs")
    
    def get_user_startup_path(self, username=None):
        """Get the user startup folder path"""
        if username is None:
            username = os.getenv('USERNAME')
        
        if username:
            return os.path.join(os.environ['APPDATA'], 
                              'Microsoft', 'Windows', 'Start Menu', 'Programs', 'Startup')
        else:
            return os.path.expanduser(r"~\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup")
    
    def create_startup_shortcut(self, source_lnk, target_dir, shortcut_name=None):
        """Create a startup shortcut in the target directory"""
        try:
            if not os.path.exists(source_lnk):
                self.logger.warning(f"Source shortcut not found: {source_lnk}")
                return False
            
            if shortcut_name is None:
                shortcut_name = os.path.basename(source_lnk)
            
            target_lnk = os.path.join(target_dir, shortcut_name)
            
            # Create target directory if it doesn't exist
            os.makedirs(target_dir, exist_ok=True)
            
            # Copy the shortcut
            shutil.copy2(source_lnk, target_lnk)
            self.logger.info(f"Created startup shortcut: {target_lnk}")
            return True
            
        except Exception as e:
            self.logger.error(f"Error creating startup shortcut: {e}")
            return False
    
    def remove_startup_shortcut(self, target_dir, shortcut_name):
        """Remove a startup shortcut from the target directory"""
        try:
            target_lnk = os.path.join(target_dir, shortcut_name)
            if os.path.exists(target_lnk):
                os.remove(target_lnk)
                self.logger.info(f"Removed startup shortcut: {target_lnk}")
                return True
            else:
                self.logger.info(f"Startup shortcut not found: {target_lnk}")
                return False
        except Exception as e:
            self.logger.error(f"Error removing startup shortcut: {e}")
            return False
    
    def setup_startup_menu(self, enable=True):
        """Setup or remove Start Menu shortcuts"""
        if not self.config.has_section('Startup'):
            self.logger.warning("Startup section not found in config")
            return False
        
        start_menu_path = self.get_startup_menu_path()
        silentwitness_menu = os.path.join(start_menu_path, "SilentWitness")
        
        if enable:
            # Create SilentWitness folder in Start Menu
            os.makedirs(silentwitness_menu, exist_ok=True)
            
            # Add shortcuts based on config
            shortcuts_added = 0
            
            if self.config.getboolean('Startup', 'startup_recorder', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder.lnk"),
                    silentwitness_menu,
                    "SilentWitness Recorder.lnk"
                ):
                    shortcuts_added += 1
            
            if self.config.getboolean('Startup', 'startup_status_gui', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder Status.lnk"),
                    silentwitness_menu,
                    "SilentWitness Status.lnk"
                ):
                    shortcuts_added += 1
            
            if self.config.getboolean('Startup', 'startup_config_gui', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder Config.lnk"),
                    silentwitness_menu,
                    "SilentWitness Config.lnk"
                ):
                    shortcuts_added += 1
            
            self.logger.info(f"Added {shortcuts_added} shortcuts to Start Menu: {silentwitness_menu}")
            return shortcuts_added > 0
            
        else:
            # Remove SilentWitness folder from Start Menu
            if os.path.exists(silentwitness_menu):
                try:
                    shutil.rmtree(silentwitness_menu)
                    self.logger.info(f"Removed Start Menu folder: {silentwitness_menu}")
                    return True
                except Exception as e:
                    self.logger.error(f"Error removing Start Menu folder: {e}")
                    return False
            return True
    
    def setup_auto_start(self, enable=True, username=None):
        """Setup or remove auto-start shortcuts for the specified user"""
        if not self.config.has_section('Startup'):
            self.logger.warning("Startup section not found in config")
            return False
        
        if username is None:
            username = self.config.get('Startup', 'startup_user', fallback='%USERNAME%', raw=True)
            if username == '%USERNAME%':
                username = os.getenv('USERNAME')
        
        user_startup_path = self.get_user_startup_path(username)
        
        if enable:
            # Add auto-start shortcuts
            shortcuts_added = 0
            
            if self.config.getboolean('Startup', 'startup_recorder', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder.lnk"),
                    user_startup_path,
                    "SilentWitness Recorder.lnk"
                ):
                    shortcuts_added += 1
            
            if self.config.getboolean('Startup', 'startup_status_gui', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder Status.lnk"),
                    user_startup_path,
                    "SilentWitness Status.lnk"
                ):
                    shortcuts_added += 1
            
            if self.config.getboolean('Startup', 'startup_config_gui', fallback=False):
                if self.create_startup_shortcut(
                    os.path.join(self.startup_dir, "Recorder Config.lnk"),
                    user_startup_path,
                    "SilentWitness Config.lnk"
                ):
                    shortcuts_added += 1
            
            self.logger.info(f"Added {shortcuts_added} auto-start shortcuts for user {username}: {user_startup_path}")
            return shortcuts_added > 0
            
        else:
            # Remove auto-start shortcuts
            shortcuts_removed = 0
            
            for shortcut in ["SilentWitness Recorder.lnk", "SilentWitness Status.lnk", "SilentWitness Config.lnk"]:
                if self.remove_startup_shortcut(user_startup_path, shortcut):
                    shortcuts_removed += 1
            
            self.logger.info(f"Removed {shortcuts_removed} auto-start shortcuts for user {username}")
            return shortcuts_removed > 0
    
    def apply_startup_config(self):
        """Apply all startup configuration settings"""
        if not self.config.has_section('Startup'):
            self.logger.warning("Startup section not found in config")
            return False
        
        try:
            # Setup Start Menu shortcuts
            if self.config.getboolean('Startup', 'enable_startup_menu', fallback=False):
                self.setup_startup_menu(True)
            else:
                self.setup_startup_menu(False)
            
            # Setup auto-start shortcuts
            if self.config.getboolean('Startup', 'enable_auto_start', fallback=False):
                self.setup_auto_start(True)
            else:
                self.setup_auto_start(False)
            
            self.logger.info("Startup configuration applied successfully")
            return True
            
        except Exception as e:
            self.logger.error(f"Error applying startup config: {e}")
            return False
    
    def get_startup_status(self):
        """Get current startup configuration status"""
        if not self.config.has_section('Startup'):
            print("WARNING: Startup section not found in config")
            return {}
        
        try:
            status = {
                'startup_menu_enabled': self.config.getboolean('Startup', 'enable_startup_menu', fallback=False),
                'auto_start_enabled': self.config.getboolean('Startup', 'enable_auto_start', fallback=False),
                'startup_user': self.config.get('Startup', 'startup_user', fallback='%USERNAME%', raw=True),
                'startup_delay': self.config.getint('Startup', 'startup_delay', fallback=10),
                'startup_recorder': self.config.getboolean('Startup', 'startup_recorder', fallback=False),
                'startup_status_gui': self.config.getboolean('Startup', 'startup_status_gui', fallback=False),
                'startup_config_gui': self.config.getboolean('Startup', 'startup_config_gui', fallback=False)
            }
            
            # Check actual file existence
            start_menu_path = self.get_startup_menu_path()
            user_startup_path = self.get_user_startup_path()
            
            status['start_menu_exists'] = os.path.exists(os.path.join(start_menu_path, "SilentWitness"))
            status['user_startup_exists'] = any(
                os.path.exists(os.path.join(user_startup_path, f)) 
                for f in ["SilentWitness Recorder.lnk", "SilentWitness Status.lnk", "SilentWitness Config.lnk"]
            )
            
            return status
        except Exception as e:
            print(f"Error getting startup status: {e}")
            return {}

def main():
    """Main function for command-line usage"""
    import argparse
    
    parser = argparse.ArgumentParser(description="SilentWitness Startup Manager")
    parser.add_argument('--config', help='Path to config.ini file')
    parser.add_argument('--startup-menu', action='store_true', help='Enable Start Menu shortcuts')
    parser.add_argument('--no-startup-menu', action='store_true', help='Disable Start Menu shortcuts')
    parser.add_argument('--auto-start', action='store_true', help='Enable auto-start shortcuts')
    parser.add_argument('--no-auto-start', action='store_true', help='Disable auto-start shortcuts')
    parser.add_argument('--apply', action='store_true', help='Apply current config settings')
    parser.add_argument('--status', action='store_true', help='Show current startup status')
    
    args = parser.parse_args()
    
    manager = StartupManager(args.config)
    
    if args.startup_menu:
        manager.setup_startup_menu(True)
    elif args.no_startup_menu:
        manager.setup_startup_menu(False)
    elif args.auto_start:
        manager.setup_auto_start(True)
    elif args.no_auto_start:
        manager.setup_auto_start(False)
    elif args.apply:
        manager.apply_startup_config()
    elif args.status:
        status = manager.get_startup_status()
        print("Startup Configuration Status:")
        for key, value in status.items():
            print(f"  {key}: {value}")
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
