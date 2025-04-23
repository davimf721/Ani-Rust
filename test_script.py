#!/usr/bin/env python3
import subprocess
import sys
import os
import time

def run_command(command):
    print(f"Executando: {' '.join(command)}")
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Capturar saída em tempo real
        while process.poll() is None:
            stdout_line = process.stdout.readline()
            if stdout_line:
                print(f"STDOUT: {stdout_line.strip()}")
            
            stderr_line = process.stderr.readline()
            if stderr_line:
                print(f"STDERR: {stderr_line.strip()}")
        
        # Capturar qualquer saída restante
        stdout, stderr = process.communicate()
        if stdout:
            print(f"STDOUT: {stdout.strip()}")
        if stderr:
            print(f"STDERR: {stderr.strip()}")
        
        return process.returncode == 0
    except Exception as e:
        print(f"Erro ao executar comando: {e}")
        return False

def check_dependencies():
    print("Verificando dependências...")
    
    # Verificar MPV
    if run_command(["which", "mpv"]):
        print("✅ MPV encontrado")
    else:
        print("❌ MPV não encontrado")
        print("Tentando instalar MPV...")
        run_command(["sudo", "apt", "update"])
        run_command(["sudo", "apt", "install", "-y", "mpv"])
    
    # Verificar curl
    if run_command(["which", "curl"]):
        print("✅ curl encontrado")
    else:
        print("❌ curl não encontrado")
        print("Tentando instalar curl...")
        run_command(["sudo", "apt", "update"])
        run_command(["sudo", "apt", "install", "-y", "curl"])
    
    # Verificar ffplay
    if run_command(["which", "ffplay"]):
        print("✅ ffplay encontrado")
    else:
        print("❌ ffplay não encontrado")
        print("Tentando instalar ffmpeg...")
        run_command(["sudo", "apt", "update"])
        run_command(["sudo", "apt", "install", "-y", "ffmpeg"])

def test_anime_playback():
    print("\n=== Testando reprodução de anime ===\n")
    
    # Verificar se o programa está compilado
    if not os.path.exists("target/debug/anirust") and not os.path.exists("target/release/anirust"):
        print("Compilando o programa...")
        if not run_command(["cargo", "build"]):
            print("❌ Falha ao compilar o programa")
            return False
    
    # Executar o programa com os parâmetros para Frieren episódio 4
    print("\nExecutando programa para reproduzir Frieren episódio 4...")
    success = run_command([
        "cargo", "run", "--", 
        "--quality", "best",
        "--episode", "4",
        "frieren"
    ])
    
    if success:
        print("\n✅ Teste concluído com sucesso!")
        return True
    else:
        print("\n❌ Teste falhou")
        return False

if __name__ == "__main__":
    print("=== Teste de reprodução do Ani-Rust ===")
    check_dependencies()
    test_anime_playback()
