#!/usr/bin/env python3
"""
Create various LMDB test databases for comprehensive testing
"""

import os
import sys
import lmdb
import json
import struct
from pathlib import Path

def create_unnamed_database(path):
    """Create an LMDB with data in the unnamed database"""
    print(f"Creating unnamed database at: {path}")
    
    # Create environment
    env = lmdb.open(str(path), max_dbs=0, map_size=10*1024*1024)  # 10MB
    
    with env.begin(write=True) as txn:
        # Add various types of data
        txn.put(b'key1', b'value1')
        txn.put(b'key2', b'value2')
        txn.put(b'binary_key', struct.pack('>I', 12345))
        txn.put(b'json_data', json.dumps({"test": "data"}).encode())
        txn.put(b'unicode_key', 'Hello 世界 🌍'.encode('utf-8'))
        
    env.close()
    print(f"✅ Created unnamed database with 5 entries")

def create_named_databases(path):
    """Create an LMDB with multiple named databases"""
    print(f"Creating named databases at: {path}")
    
    # Create environment with space for named databases
    env = lmdb.open(str(path), max_dbs=10, map_size=10*1024*1024)
    
    # Create multiple named databases
    with env.begin(write=True) as txn:
        # Database 1: users
        users_db = env.open_db(b'users', txn=txn)
        txn.put(b'alice', b'alice@example.com', db=users_db)
        txn.put(b'bob', b'bob@example.com', db=users_db)
        
        # Database 2: config
        config_db = env.open_db(b'config', txn=txn)
        txn.put(b'version', b'1.0.0', db=config_db)
        txn.put(b'debug', b'false', db=config_db)
        
        # Database 3: logs
        logs_db = env.open_db(b'logs', txn=txn)
        txn.put(b'2024-01-01', b'System started', db=logs_db)
        txn.put(b'2024-01-02', b'User logged in', db=logs_db)
    
    env.close()
    print(f"✅ Created 3 named databases: users, config, logs")

def create_empty_database(path):
    """Create an empty LMDB environment"""
    print(f"Creating empty database at: {path}")
    
    env = lmdb.open(str(path), max_dbs=0, map_size=1024*1024)  # 1MB
    env.close()
    print(f"✅ Created empty database")

def create_large_database(path):
    """Create a larger database for performance testing"""
    print(f"Creating large database at: {path}")
    
    env = lmdb.open(str(path), max_dbs=0, map_size=50*1024*1024)  # 50MB
    
    with env.begin(write=True) as txn:
        # Add 1000 entries
        for i in range(1000):
            key = f'key_{i:04d}'.encode()
            value = f'value_{i:04d}_' + ('x' * 100)  # ~100 byte values
            txn.put(key, value.encode())
    
    env.close()
    print(f"✅ Created large database with 1000 entries")

def create_mixed_database(path):
    """Create a database with both unnamed and named databases"""
    print(f"Creating mixed database at: {path}")
    
    env = lmdb.open(str(path), max_dbs=5, map_size=10*1024*1024)
    
    with env.begin(write=True) as txn:
        # Add data to unnamed database
        txn.put(b'root_key1', b'root_value1')
        txn.put(b'root_key2', b'root_value2')
        
        # Create a named database
        named_db = env.open_db(b'named_db', txn=txn)
        txn.put(b'named_key1', b'named_value1', db=named_db)
        txn.put(b'named_key2', b'named_value2', db=named_db)
    
    env.close()
    print(f"✅ Created mixed database with unnamed and named data")

def main():
    """Create all test databases"""
    base_dir = Path("test_data")
    base_dir.mkdir(exist_ok=True)
    
    try:
        # Create various test databases
        create_unnamed_database(base_dir / "unnamed_db")
        create_named_databases(base_dir / "named_dbs")
        create_empty_database(base_dir / "empty_db")
        create_large_database(base_dir / "large_db")
        create_mixed_database(base_dir / "mixed_db")
        
        print("\n✅ All test databases created successfully!")
        print(f"📁 Test databases location: {base_dir.absolute()}")
        
    except ImportError:
        print("❌ Error: lmdb module not installed")
        print("Install with: pip install lmdb")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error creating databases: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()