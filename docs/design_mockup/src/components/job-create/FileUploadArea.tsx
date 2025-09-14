import { useCallback } from "react";
import { UploadedFile } from "./CreateJobView";
import { Button } from "../ui/button";
import { Card, CardContent } from "../ui/card";
import { Upload, X, FileText } from "lucide-react";

interface FileUploadAreaProps {
  uploadedFiles: UploadedFile[];
  onChange: (files: UploadedFile[]) => void;
  error?: string;
}

export function FileUploadArea({ uploadedFiles, onChange, error }: FileUploadAreaProps) {
  const acceptedExtensions = [".pdb", ".psf", ".prm"];

  const handleFileSelect = useCallback((files: FileList | null) => {
    if (!files) return;

    const newFiles: UploadedFile[] = [];
    
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const extension = "." + file.name.split('.').pop()?.toLowerCase();
      
      if (acceptedExtensions.includes(extension)) {
        newFiles.push({
          name: file.name,
          size: file.size,
          type: extension,
          file: file
        });
      }
    }

    onChange([...uploadedFiles, ...newFiles]);
  }, [uploadedFiles, onChange]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    handleFileSelect(e.dataTransfer.files);
  }, [handleFileSelect]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  const handleBrowseClick = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.accept = acceptedExtensions.join(',');
    input.onchange = (e) => {
      const target = e.target as HTMLInputElement;
      handleFileSelect(target.files);
    };
    input.click();
  };

  const removeFile = (index: number) => {
    const newFiles = uploadedFiles.filter((_, i) => i !== index);
    onChange(newFiles);
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-4">
      {/* Upload Area */}
      <Card
        className={`border-2 border-dashed cursor-pointer transition-colors hover:bg-muted/50 ${
          error ? "border-destructive" : "border-muted-foreground/25"
        }`}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onClick={handleBrowseClick}
      >
        <CardContent className="p-8 text-center">
          <Upload className="w-8 h-8 mx-auto mb-4 text-muted-foreground" />
          <div className="space-y-2">
            <div className="text-lg">Drag & drop files here or</div>
            <Button variant="outline" onClick={handleBrowseClick}>
              Click to Browse
            </Button>
          </div>
          <div className="mt-4 text-sm text-muted-foreground">
            Accepted: {acceptedExtensions.join(", ")}
          </div>
        </CardContent>
      </Card>

      {error && (
        <div className="text-sm text-destructive">{error}</div>
      )}

      {/* Uploaded Files List */}
      {uploadedFiles.length > 0 && (
        <div className="space-y-2">
          <h4 className="font-medium">Uploaded Files:</h4>
          
          {uploadedFiles.map((file, index) => (
            <div key={index} className="flex items-center gap-3 p-3 bg-muted/30 rounded-lg">
              <FileText className="w-4 h-4 text-muted-foreground" />
              
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium truncate">{file.name}</div>
                <div className="text-xs text-muted-foreground">
                  {formatFileSize(file.size)} • {file.type.toUpperCase()} file
                </div>
              </div>
              
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  removeFile(index);
                }}
                className="h-8 w-8 p-0"
              >
                <X className="w-4 h-4" />
              </Button>
            </div>
          ))}
        </div>
      )}

      {/* File Requirements */}
      <div className="text-sm text-muted-foreground">
        <div className="font-medium mb-1">Required files:</div>
        <ul className="list-disc list-inside space-y-1">
          <li>Structure file (.pdb or .psf)</li>
          <li>Parameter file(s) (.prm)</li>
          <li>Additional parameter files as needed</li>
        </ul>
      </div>
    </div>
  );
}