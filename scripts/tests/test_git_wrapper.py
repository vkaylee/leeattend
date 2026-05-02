import os
import sys
import tempfile
from unittest import mock
import pytest

# Import the wrapper module
import _git_wrapper

def test_non_commit_command(monkeypatch):
    """Test git command other than commit passes through untouched."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'status', '--short'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit') as mock_exit:
        _git_wrapper.main()
        
    # Check that subprocess.call was called with correct arguments
    called_cmd = mock_call.call_args[0][0]
    assert 'git' in called_cmd
    assert 'status' in called_cmd
    assert '--short' in called_cmd
    assert '-F' not in called_cmd
    
    mock_exit.assert_called_once_with(0)

def test_commit_without_message(monkeypatch):
    """Test git commit without -m."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-a'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit') as mock_exit:
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert 'commit' in called_cmd
    assert '-a' in called_cmd
    assert '-F' not in called_cmd
    mock_exit.assert_called_once_with(0)

def test_commit_with_space_separated_m(monkeypatch):
    """Test git commit -m 'message'."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-a', '-m', 'Hello world'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit') as mock_exit:
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert 'commit' in called_cmd
    assert '-a' in called_cmd
    assert '-m' not in called_cmd
    assert 'Hello world' not in called_cmd
    
    # Verify -F is injected and file contains the message
    assert '-F' in called_cmd
    f_index = called_cmd.index('-F')
    temp_file = called_cmd[f_index + 1]
    
    assert os.path.exists(temp_file)
    with open(temp_file, 'r', encoding='utf-8') as f:
        content = f.read()
    assert content == "Hello world\n"
    
    # Manual cleanup as we're mocking exit
    os.remove(temp_file)

def test_commit_with_attached_m(monkeypatch):
    """Test git commit -m'message'."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-mFix issue'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert '-F' in called_cmd
    temp_file = called_cmd[called_cmd.index('-F') + 1]
    
    with open(temp_file, 'r', encoding='utf-8') as f:
        assert f.read() == "Fix issue\n"
    os.remove(temp_file)

def test_commit_with_attached_message(monkeypatch):
    """Test git commit --message='message'."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '--message=Hello'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert '-F' in called_cmd
    temp_file = called_cmd[called_cmd.index('-F') + 1]
    
    with open(temp_file, 'r', encoding='utf-8') as f:
        assert f.read() == "Hello\n"
    os.remove(temp_file)

def test_commit_with_multiple_messages(monkeypatch):
    """Test git commit with multiple -m."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-m', 'Title', '--message', 'Body'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert '-F' in called_cmd
    temp_file = called_cmd[called_cmd.index('-F') + 1]
    
    with open(temp_file, 'r', encoding='utf-8') as f:
        assert f.read() == "Title\n\nBody\n"
    os.remove(temp_file)

def test_atexit_cleanup(monkeypatch):
    """Test that the cleanup function correctly deletes files, and ignores OSErrors."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-m', 'Msg'])
    
    # Spy on atexit.register to capture the cleanup function
    cleanup_func = None
    import atexit as atexit_module
    original_register = atexit_module.register
    
    def fake_register(func, *args, **kwargs):
        nonlocal cleanup_func
        cleanup_func = func
        return original_register(func, *args, **kwargs)
        
    with mock.patch('atexit.register', side_effect=fake_register), \
         mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    assert cleanup_func is not None
    called_cmd = mock_call.call_args[0][0]
    temp_file = called_cmd[called_cmd.index('-F') + 1]
    
    # 1. Normal cleanup
    assert os.path.exists(temp_file)
    cleanup_func()
    assert not os.path.exists(temp_file)
    
    # 2. Ignore OSError cleanup (file already deleted)
    # This shouldn't raise any exception
    cleanup_func()

def test_commit_auto_inject_no_gpg_sign(monkeypatch):
    """Test that --no-gpg-sign is automatically injected."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-m', 'Msg'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert '--no-gpg-sign' in called_cmd

def test_commit_with_existing_no_gpg_sign(monkeypatch):
    """Test that --no-gpg-sign is not injected twice."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '--no-gpg-sign', '-m', 'Msg'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert called_cmd.count('--no-gpg-sign') == 1

def test_commit_with_s_flag(monkeypatch):
    """Test that --no-gpg-sign is not injected if -S or --gpg-sign is present."""
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '-S', '-m', 'Msg'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    assert '--no-gpg-sign' not in called_cmd
    
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'commit', '--gpg-sign', '-m', 'Msg'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call2, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd2 = mock_call2.call_args[0][0]
    assert '--no-gpg-sign' not in called_cmd2

def test_timeout_env_var(monkeypatch):
    """Test GIT_TIMEOUT env var."""
    monkeypatch.setenv('GIT_TIMEOUT', '300')
    monkeypatch.setattr(sys, 'argv', ['_git_wrapper.py', 'status'])
    
    with mock.patch('subprocess.call', return_value=0) as mock_call, \
         mock.patch('sys.exit'):
        _git_wrapper.main()
        
    called_cmd = mock_call.call_args[0][0]
    # In the command constructed: [sys.executable, safe_run_path, timeout, 'git']
    assert '300' in called_cmd
